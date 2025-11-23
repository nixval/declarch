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
    pub target: Option<String>,
    pub noconfirm: bool,
    pub only_aur: bool,
    pub only_flatpak: bool,
}

pub fn run(options: SyncOptions) -> Result<()> {
    output::header("Synchronizing Packages");

    // 0. Determine Sync Target
    let sync_target = if let Some(t) = &options.target {
        SyncTarget::Named(t.clone())
    } else if options.only_aur {
        SyncTarget::Backend(Backend::Aur)
    } else if options.only_flatpak {
        SyncTarget::Backend(Backend::Flatpak)
    } else {
        SyncTarget::All
    };

    if let SyncTarget::Named(ref s) = sync_target {
        output::info(&format!("Targeting specific scope: {}", s.cyan()));
        if options.prune {
            output::warning("Pruning is disabled when using --target for safety.");
        }
    }

    // 1. System Update
    if options.update {
        output::info("Updating system...");
        if !options.dry_run {
            // Detect helper
            let helper = if which::which("paru").is_ok() { "paru" } else { "yay" };
            let mut cmd = Command::new(helper);
            cmd.arg("-Syu");
            if options.yes || options.noconfirm { cmd.arg("--noconfirm"); }
            
            let status = cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).status()?;
            if !status.success() { return Err(DeclarchError::Other("System update failed".into())); }
        }
    }

    // 2. Load Managers & Batch Fetch Installed Packages
    output::info("Scanning system state...");
    let mut installed_snapshot: HashMap<PackageId, PackageMetadata> = HashMap::new();
    let mut managers: HashMap<Backend, Box<dyn PackageManager>> = HashMap::new();

    // Init Managers
    managers.insert(Backend::Aur, Box::new(AurManager::new()));
    managers.insert(Backend::Flatpak, Box::new(FlatpakManager));

    // Fetch Data
    for (backend, mgr) in &managers {
        if !mgr.is_available() { continue; }
        
        let packages = mgr.list_installed()?;
        for (name, meta) in packages {
            let id = PackageId { name, backend: backend.clone() };
            installed_snapshot.insert(id, meta);
        }
    }

    // 3. Load Config & State
    let state = state::io::load_state()?;
    let config_path = paths::config_file()?;
    let config = loader::load_root_config(&config_path)?;

    // 4. Resolve
    let tx = resolver::resolve(&config, &state, &installed_snapshot, &sync_target)?;

    // 5. Preview
    output::separator();
    if !tx.to_install.is_empty() {
        println!("{}", "To Install:".green().bold());
        for pkg in &tx.to_install { println!("  + {}", pkg); }
    }
    if !tx.to_adopt.is_empty() {
        println!("{}", "To Adopt (Track in State):".yellow().bold());
        for pkg in &tx.to_adopt { println!("  ~ {}", pkg); }
    }
    
    let should_prune = options.prune && matches!(sync_target, SyncTarget::All);
    
    if !tx.to_prune.is_empty() {
        let header = if should_prune { "To Remove:".red().bold() } else { "To Remove (Skipped):".dimmed() };
        println!("{}", header);
        for pkg in &tx.to_prune { 
            if should_prune { println!("  - {}", pkg); } 
            else { println!("  - {}", pkg.to_string().dimmed()); }
        }
    }

    if tx.to_install.is_empty() && tx.to_adopt.is_empty() && (!should_prune || tx.to_prune.is_empty()) {
        output::success("Nothing to do.");
        return Ok(());
    }

    if options.dry_run { return Ok(()); }

    // 6. Execute
    if !options.yes && !options.noconfirm {
        if !output::prompt_yes_no("Proceed?") { return Err(DeclarchError::Interrupted); }
    }

    // Grouping by backend for batch execution
    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install {
        installs.entry(pkg.backend).or_default().push(pkg.name.clone());
    }

    let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
    if should_prune {
        for pkg in tx.to_prune {
            removes.entry(pkg.backend).or_default().push(pkg.name.clone());
        }
    }

    // Execute Installs
    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend.to_string()));
            mgr.install(&pkgs)?;
        }
    }

    // Execute Removes
    for (backend, pkgs) in removes {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Removing {} packages...", backend.to_string()));
            mgr.remove(&pkgs)?;
        }
    }

    let mut new_state = state.clone();
    
    for pkg in tx.to_adopt {
        new_state.packages.insert(pkg.name, PackageState { 
            backend: match pkg.backend { Backend::Aur => StateBackend::Aur, Backend::Flatpak => StateBackend::Flatpak },
            installed_at: Utc::now(),
            version: None 
        });
    }

    state::io::save_state(&new_state)?;
    output::success("Sync complete!");

    Ok(())
}
