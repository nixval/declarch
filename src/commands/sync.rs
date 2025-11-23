use crate::utils::paths;
use crate::ui as output;
use crate::state::{self, types::{Backend as StateBackend, PackageState}};
use crate::config::loader;
use crate::core::{resolver, types::{Backend, PackageId, PackageMetadata, SyncTarget}};
use crate::packages::{PackageManager, aur::AurManager, flatpak::FlatpakManager};
use crate::error::{DeclarchError, Result};
use colored::Colorize;
use chrono::Utc;
use std::collections::HashMap;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
    pub gc: bool,
    pub update: bool,
    pub yes: bool,
    pub force: bool,
    pub target: Option<String>,
    pub noconfirm: bool,
}

pub fn run(options: SyncOptions) -> Result<()> {
    output::header("Synchronizing Packages");

    let sync_target = if let Some(t) = &options.target {
        match t.to_lowercase().as_str() {
            "aur" | "repo" | "paru" | "pacman" => SyncTarget::Backend(Backend::Aur),
            "flatpak" => SyncTarget::Backend(Backend::Flatpak),
            _ => SyncTarget::Named(t.clone()),
        }
    } else {
        SyncTarget::All
    };

    let is_partial_sync = !matches!(sync_target, SyncTarget::All);
    
    if is_partial_sync {
        if let SyncTarget::Named(ref s) = sync_target {
            output::info(&format!("Targeting specific scope: {}", s.cyan()));
        }
        
        if options.prune {
            if options.force {
                output::warning("FORCE MODE: Pruning enabled despite partial sync target.");
            } else {
                output::warning("Pruning is disabled when using --target for safety.");
                output::info("Use --force to override this safety check.");
            }
        }
    }

    if options.update {
        output::info("Updating system...");
        if !options.dry_run {
            let helper = if which::which("paru").is_ok() { "paru" } else { "yay" };
            let mut cmd = Command::new(helper);
            cmd.arg("-Syu");
            if options.yes || options.noconfirm { cmd.arg("--noconfirm"); }
            
            let status = cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).status()?;
            if !status.success() { return Err(DeclarchError::Other("System update failed".into())); }
        }
    }

    output::info("Scanning system state...");
    let mut installed_snapshot: HashMap<PackageId, PackageMetadata> = HashMap::new();
    let mut managers: HashMap<Backend, Box<dyn PackageManager>> = HashMap::new();

    managers.insert(Backend::Aur, Box::new(AurManager::new(options.noconfirm)));
    managers.insert(Backend::Flatpak, Box::new(FlatpakManager::new(options.noconfirm)));

    for (backend, mgr) in &managers {
        if !mgr.is_available() { continue; }
        
        let packages = mgr.list_installed()?;
        for (name, meta) in packages {
            let id = PackageId { name, backend: backend.clone() };
            installed_snapshot.insert(id, meta);
        }
    }

    let mut state = state::io::load_state()?;
    let config_path = paths::config_file()?;
    let config = loader::load_root_config(&config_path)?;

    let duplicates = config.get_duplicates();
    if !duplicates.is_empty() {
        println!("  ℹ {} duplicate packages merged automatically.", duplicates.len().to_string().yellow());
    }

    let tx = resolver::resolve(&config, &state, &installed_snapshot, &sync_target)?;

    output::separator();
    if !tx.to_install.is_empty() {
        println!("{}", "To Install:".green().bold());
        for pkg in &tx.to_install { println!("  + {}", pkg); }
    }
    
    if !tx.to_update_meta.is_empty() && options.dry_run {
        println!("{}", "State Updates (Drift detected):".blue().bold());
        for pkg in &tx.to_update_meta { 
            if let Some(meta) = installed_snapshot.get(pkg) {
                println!("  ~ {} (v{})", pkg, meta.version.as_deref().unwrap_or("?")); 
            }
        }
    }

    if !tx.to_adopt.is_empty() {
        println!("{}", "To Adopt (Track in State):".yellow().bold());
        for pkg in &tx.to_adopt { println!("  ~ {}", pkg); }
    }
    
    let allow_prune = matches!(sync_target, SyncTarget::All) || options.force;
    let should_prune = options.prune && allow_prune;
    
    if !tx.to_prune.is_empty() {
        let header = if should_prune { "To Remove:".red().bold() } else { "To Remove (Skipped):".dimmed() };
        println!("{}", header);
        for pkg in &tx.to_prune { 
            if should_prune { println!("  - {}", pkg); } 
            else { println!("  - {}", pkg.to_string().dimmed()); }
        }
    }

    if tx.to_install.is_empty() && tx.to_adopt.is_empty() && tx.to_update_meta.is_empty() && (!should_prune || tx.to_prune.is_empty()) {
        output::success("System is in sync.");
        return Ok(());
    }

    if options.dry_run { return Ok(()); }

    let skip_prompt = options.yes || options.noconfirm || options.force;
    
    if !skip_prompt {
        if !output::prompt_yes_no("Proceed?") { return Err(DeclarchError::Interrupted); }
    }

    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install.iter() {
        installs.entry(pkg.backend.clone()).or_default().push(pkg.name.clone());
    }

    let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
    if should_prune {
        for pkg in tx.to_prune.iter() {
            removes.entry(pkg.backend.clone()).or_default().push(pkg.name.clone());
        }
    }

    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend.to_string()));
            mgr.install(&pkgs)?;
        }
    }

    for (backend, pkgs) in removes {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Removing {} packages...", backend.to_string()));
            mgr.remove(&pkgs)?;
        }
    }

    let mut to_upsert = tx.to_install.clone();
    to_upsert.extend(tx.to_adopt.clone());
    to_upsert.extend(tx.to_update_meta.clone());

    for pkg in to_upsert {
        let version = installed_snapshot.get(&pkg).and_then(|m| m.version.clone());
        
        state.packages.insert(pkg.name, PackageState { 
            backend: match pkg.backend {
                Backend::Aur => StateBackend::Aur,
                Backend::Flatpak => StateBackend::Flatpak,
            },
            installed_at: Utc::now(),
            version,
        });
    }

    if should_prune {
        for pkg in tx.to_prune {
            state.packages.remove(&pkg.name);
        }
    }

    state.meta.last_sync = Utc::now();
    state::io::save_state(&state)?;
    
    output::success("Sync complete!");

    Ok(())
}
