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

// --- SAFETY NET EXPANDED (The Doomsday List) ---
const CRITICAL_PACKAGES: &[&str] = &[
    "linux", "linux-lts", "linux-zen", "linux-hardened", 
    "linux-api-headers", "linux-firmware", 
    "amd-ucode", "intel-ucode",
    "grub", "systemd-boot", "efibootmgr", "os-prober",
    "base", "base-devel", 
    "systemd", "systemd-libs", "systemd-sysvcompat",
    "glibc", "gcc-libs", "zlib", "openssl", "readline",
    "bash", "zsh", "fish", "sh",
    "sudo", "doas", "pam", "shadow", "util-linux", "coreutils",
    "pacman", "pacman-contrib", "archlinux-keyring", 
    "paru", "yay", "aura", "pikaur",
    "flatpak", 
    "declarch", "declarch-bin", "git", "curl", "wget", "tar",
    "mesa", "nvidia", "nvidia-utils", "nvidia-dkms",
    "networkmanager", "iwd", "wpa_supplicant",
    "btrfs-progs", "e2fsprogs", "dosfstools", "ntfs-3g"
];

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

    // 1. Target Resolution
    let sync_target = if let Some(t) = &options.target {
        match t.to_lowercase().as_str() {
            "aur" | "repo" | "paru" | "pacman" => SyncTarget::Backend(Backend::Aur),
            "flatpak" => SyncTarget::Backend(Backend::Flatpak),
            _ => SyncTarget::Named(t.clone()),
        }
    } else {
        SyncTarget::All
    };

    // 2. Load Config
    let config_path = paths::config_file()?;
    let config = loader::load_root_config(&config_path)?;
    
    // 3. System Update
    let global_config = crate::config::types::GlobalConfig::default(); 
    let aur_helper = global_config.aur_helper.to_string();

    if options.update {
        output::info("Updating system...");
        if !options.dry_run {
            let mut cmd = Command::new(&aur_helper);
            cmd.arg("-Syu");
            if options.yes || options.noconfirm { cmd.arg("--noconfirm"); }
            
            let status = cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).status()?;
            if !status.success() { return Err(DeclarchError::Other("System update failed".into())); }
        }
    }

    // 4. Initialize Managers & Snapshot
    output::info("Scanning system state...");
    let mut installed_snapshot: HashMap<PackageId, PackageMetadata> = HashMap::new();
    let mut managers: HashMap<Backend, Box<dyn PackageManager>> = HashMap::new();

    managers.insert(Backend::Aur, Box::new(AurManager::new(aur_helper, options.noconfirm)));
    managers.insert(Backend::Flatpak, Box::new(FlatpakManager::new(options.noconfirm)));

    for (backend, mgr) in &managers {
        if !mgr.is_available() {
            if matches!(sync_target, SyncTarget::Backend(ref b) if b == backend) {
                 output::warning(&format!("Backend '{}' is not available on this system.", backend));
            }
            continue; 
        }
        
        let packages = mgr.list_installed()?;
        for (name, meta) in packages {
            let id = PackageId { name, backend: backend.clone() };
            installed_snapshot.insert(id, meta);
        }
    }

    // 5. Load State & Resolve
    let mut state = state::io::load_state()?;
    let tx = resolver::resolve(&config, &state, &installed_snapshot, &sync_target)?;

    // --- SMART PARTIAL UPGRADE CHECK ---
    // Only warn if:
    // 1. We are installing new things
    // 2. We are NOT updating in this run
    // 3. Last update was > 24 hours ago (or never)
    if !options.update && !tx.to_install.is_empty() {
        let should_warn = match state.meta.last_update {
            Some(last) => {
                let hours = Utc::now().signed_duration_since(last).num_hours();
                hours > 24 
            },
            None => true, // Warn if we have no record of update
        };

        if should_warn {
            // Short, non-intrusive warning
            let time_str = state.meta.last_update
                .map(|t| format!("{}h ago", Utc::now().signed_duration_since(t).num_hours()))
                .unwrap_or("unknown".to_string());
            
            output::separator();
            println!("{} Last system update: {}. Use {} to refresh.", 
                "⚠ Partial Upgrade Risk:".yellow().bold(),
                time_str.white(),
                "--update".bold()
            );
        }
    }

    // 6. Display Plan
    output::separator();
    if !tx.to_install.is_empty() {
        println!("{}", "To Install:".green().bold());
        for pkg in &tx.to_install { println!("  + {}", pkg); }
    }
    
    if !tx.to_update_meta.is_empty() && options.dry_run {
        println!("{}", "State Updates (Drift detected):".blue().bold());
        for pkg in &tx.to_update_meta { 
             let v_display = installed_snapshot.get(pkg)
                .map(|m| m.version.as_deref().unwrap_or("?"))
                .unwrap_or("smart-match");
            println!("  ~ {} (v{})", pkg, v_display); 
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
            let is_critical = CRITICAL_PACKAGES.contains(&pkg.name.as_str());
            
            if should_prune { 
                if is_critical {
                     println!("  - {} {}", pkg.to_string().yellow(), "(Protected - Detaching from State)".italic());
                } else {
                     println!("  - {}", pkg); 
                }
            } else { 
                println!("  - {}", pkg.to_string().dimmed()); 
            }
        }
    }

    if tx.to_install.is_empty() && tx.to_adopt.is_empty() && tx.to_update_meta.is_empty() && (!should_prune || tx.to_prune.is_empty()) {
        output::success("System is in sync.");
        // Even if nothing to do, if user requested update, we should record it.
        if options.update {
            state.meta.last_update = Some(Utc::now());
            state::io::save_state(&state)?;
        }
        return Ok(());
    }

    if options.dry_run { return Ok(()); }

    // 7. Execution
    let skip_prompt = options.yes || options.noconfirm || options.force;
    if !skip_prompt {
        if !output::prompt_yes_no("Proceed?") { return Err(DeclarchError::Interrupted); }
    }

    // -- INSTALLATION --
    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install.iter() {
        installs.entry(pkg.backend.clone()).or_default().push(pkg.name.clone());
    }

    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend.to_string()));
            mgr.install(&pkgs)?;
        }
    }

    // -- REFRESH SNAPSHOT --
    if !tx.to_install.is_empty() {
        output::info("Refreshing package state...");
        for (backend, mgr) in &managers {
            if !mgr.is_available() { continue; }
            let packages = mgr.list_installed()?;
            for (name, meta) in packages {
                let id = PackageId { name, backend: backend.clone() };
                installed_snapshot.insert(id, meta);
            }
        }
    }

    // -- REMOVAL --
    if should_prune {
        let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
        
        for pkg in tx.to_prune.iter() {
            if CRITICAL_PACKAGES.contains(&pkg.name.as_str()) {
                continue;
            }
            removes.entry(pkg.backend.clone()).or_default().push(pkg.name.clone());
        }

        for (backend, pkgs) in removes {
            if !pkgs.is_empty() {
                 if let Some(mgr) = managers.get(&backend) {
                    output::info(&format!("Removing {} packages...", backend.to_string()));
                    mgr.remove(&pkgs)?;
                }
            }
        }
    }

    // 8. Update State
let mut to_upsert = tx.to_install.clone();
    to_upsert.extend(tx.to_adopt.clone());
    to_upsert.extend(tx.to_update_meta.clone());

    for pkg in to_upsert {
        let mut meta = installed_snapshot.get(&pkg);
        
        // --- REVERSE LOOKUP (Agar key config stabil tapi version diambil dari system) ---
        if meta.is_none() {
            match pkg.backend {
                Backend::Aur => {
                    // Suffix Check
                    let suffixes = ["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];
                    for suffix in suffixes {
                        let alt_name = format!("{}{}", pkg.name, suffix);
                        let alt_id = PackageId { name: alt_name, backend: Backend::Aur };
                        if let Some(m) = installed_snapshot.get(&alt_id) {
                            meta = Some(m);
                            break;
                        }
                    }
                    // Prefix Check
                    if meta.is_none() {
                        if let Some((prefix, _)) = pkg.name.split_once('-') {
                            let alt_id = PackageId { name: prefix.to_string(), backend: Backend::Aur };
                            if let Some(m) = installed_snapshot.get(&alt_id) {
                                meta = Some(m);
                            }
                        }
                    }
                },
                Backend::Flatpak => {
                    // Flatpak Fuzzy Lookup
                    let search = pkg.name.to_lowercase();
                    for (installed_id, m) in &installed_snapshot {
                        if installed_id.backend == Backend::Flatpak {
                            let installed_name = installed_id.name.to_lowercase();
                            if installed_name.contains(&search) {
                                meta = Some(m);
                                break; 
                            }
                        }
                    }
                }
            }
        }

        let version = meta.and_then(|m| m.version.clone());
        let key = format!("{}:{}", pkg.backend, pkg.name);
        
        state.packages.insert(key, PackageState { 
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
            let key = format!("{}:{}", pkg.backend, pkg.name);
            state.packages.remove(&key);
        }
    }

    state.meta.last_sync = Utc::now();
    
    // RECORD LAST UPDATE
    if options.update {
        state.meta.last_update = Some(Utc::now());
    }

    state::io::save_state(&state)?;
    
    output::success("Sync complete!");

    Ok(())
}
