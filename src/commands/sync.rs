use crate::utils::{output, errors::{DeclarchError, Result}, paths};
use crate::state::{self, types::{Backend, PackageState}};
use crate::config::{loader, types::AurHelper};
use crate::core::resolver::{self, SystemInspector};
use crate::package::factory::PackageManagerFactory;
use colored::Colorize;
use chrono::Utc;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
    pub gc: bool,
    pub update: bool,
    pub yes: bool,
}

// Implementasi "Mata Asli" yang mengecek sistem beneran
struct RealSystemInspector {
    helper: AurHelper,
}

impl SystemInspector for RealSystemInspector {
    fn is_installed(&self, pkg: &str, backend: &Backend) -> Result<bool> {
        let mgr = PackageManagerFactory::get(backend.clone(), self.helper)?;
        mgr.check(pkg)
    }
}

pub fn run(options: SyncOptions) -> Result<()> {
    output::header("Synchronizing Packages");

    // STEP 0: System Update
    if options.update {
        output::info("Updating system...");
        if !options.dry_run {
            let mut cmd = Command::new("paru");
            cmd.arg("-Syu");
            if options.yes { cmd.arg("--noconfirm"); }
            let status = cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).status()
                .map_err(|e| DeclarchError::SystemCommandFailed { command: "paru -Syu".into(), reason: e.to_string() })?;
            if !status.success() { return Err(DeclarchError::Other("System update failed".into())); }
        } else {
            output::warning("Skipping system update (dry-run)");
        }
        output::separator();
    }

    // STEP 1: Load
    let mut state = state::io::load_state()?;
    output::info(&format!("Loaded state. Last sync: {}", state.meta.last_sync));

    let config_path = paths::config_file()?;
    if !config_path.exists() { return Err(DeclarchError::ConfigNotFound { path: config_path }); }
    let merged_config = loader::load_root_config(&config_path)?;
    
    // STEP 2: Resolve
    output::info("Resolving dependencies...");
    let helper = AurHelper::Paru;
    let inspector = RealSystemInspector { helper }; 
    
    let tx = resolver::resolve(&merged_config, &state, &inspector)?;

    if tx.to_install.is_empty() && tx.to_prune.is_empty() && tx.to_adopt.is_empty() && !options.gc {
        output::success("System is in sync.");
        return Ok(());
    }

    // STEP 3: Preview
    output::separator();
    if !tx.to_install.is_empty() {
        println!("{}", "To Install:".green().bold());
        for (pkg, backend) in &tx.to_install { println!("  + {} ({:?})", pkg, backend); }
    }
    if !tx.to_adopt.is_empty() {
        println!("{}", "To Adopt:".yellow().bold());
        for (pkg, _) in &tx.to_adopt { println!("  ~ {}", pkg); }
    }
    if !tx.to_prune.is_empty() {
        let header = if options.prune { "To Remove (Prune):".red().bold() } else { "To Remove (Skipped):".red().dimmed() };
        println!("{}", header);
        for (pkg, _) in &tx.to_prune { println!("  - {}", pkg); }
    }
    output::separator();

    if options.dry_run {
        output::warning("Dry run enabled.");
        return Ok(());
    }

    // STEP 4: Execute
    if !tx.to_install.is_empty() || (options.prune && !tx.to_prune.is_empty()) {
        if !options.yes {
            if !output::prompt_yes_no("Proceed with changes?") {
                output::warning("Aborted by user.");
                return Ok(());
            }
        }

        let aur_mgr = PackageManagerFactory::get(Backend::Aur, helper)?;
        let flatpak_mgr = PackageManagerFactory::get(Backend::Flatpak, helper)?;

        for (pkg, backend) in tx.to_adopt {
            state.packages.insert(pkg, PackageState { backend, installed_at: Utc::now(), version: None });
        }

        let mut aur_pkgs = vec![];
        let mut flatpak_pkgs = vec![];
        for (pkg, backend) in tx.to_install {
            match backend {
                Backend::Aur => aur_pkgs.push(pkg.clone()),
                Backend::Flatpak => flatpak_pkgs.push(pkg.clone()),
            }
            state.packages.insert(pkg, PackageState { backend, installed_at: Utc::now(), version: None });
        }

        if !aur_pkgs.is_empty() { aur_mgr.install(&aur_pkgs)?; }
        if !flatpak_pkgs.is_empty() { flatpak_mgr.install(&flatpak_pkgs)?; }

        if options.prune && !tx.to_prune.is_empty() {
            let mut aur_prune = vec![];
            let mut flatpak_prune = vec![];
            for (pkg, backend) in &tx.to_prune {
                match backend {
                    Backend::Aur => aur_prune.push(pkg.clone()),
                    Backend::Flatpak => flatpak_prune.push(pkg.clone()),
                }
                state.packages.remove(pkg);
            }
            if !aur_prune.is_empty() { aur_mgr.remove(&aur_prune)?; }
            if !flatpak_prune.is_empty() { flatpak_mgr.remove(&flatpak_prune)?; }
        }

        state.meta.last_sync = Utc::now();
        state::io::save_state(&state)?;
        output::success("Sync complete!");
    }

    // STEP 5: GC
    if options.gc {
        output::separator();
        output::info("Checking for orphans (GC)...");
        let check = Command::new("pacman").arg("-Qdtq").output()?;
        let orphans = String::from_utf8_lossy(&check.stdout);
        let orphan_list: Vec<&str> = orphans.lines().collect();

        if !orphan_list.is_empty() {
            println!("Orphans found: {}", orphan_list.join(", ").dimmed());
            let proceed_gc = if options.yes { true } else { output::prompt_yes_no("Remove orphans?") };
            if proceed_gc {
                 let status = Command::new("sudo").arg("pacman").arg("-Rns").args(&orphan_list)
                    .stdin(Stdio::inherit()).stdout(Stdio::inherit()).status()?;
                if status.success() { output::success("Orphans removed."); }
            }
        } else {
            output::success("No orphans found.");
        }
    }
    Ok(())
}
