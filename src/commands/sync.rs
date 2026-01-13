use crate::utils::paths;
use crate::ui as output;
use crate::state::{self, types::{Backend, PackageState, State}};
use crate::config::loader;
use crate::core::{resolver, types::{PackageId, PackageMetadata, SyncTarget}};
use crate::packages::{PackageManager, aur::AurManager, flatpak::FlatpakManager};
use crate::error::{DeclarchError, Result};
use colored::Colorize;
use chrono::Utc;
use std::collections::HashMap;
use std::process::{Command, Stdio};

// --- SAFETY NET (Static Doomsday List) ---
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

    // --- VARIANT TRANSITION DETECTION ---
    // Check for variant mismatches and error early (strict approach)
    let matcher = crate::core::matcher::PackageMatcher::new();
    let mut variant_mismatches: Vec<(String, String)> = Vec::new();

    // Only check for variant transitions in full sync or when targeting specific backends
    if matches!(sync_target, SyncTarget::All | SyncTarget::Backend(_)) {
        for pkg_id in config.packages.keys() {
            // Skip if this package is already in transaction to install
            if tx.to_install.iter().any(|p| p.name == pkg_id.name) {
                continue;
            }

            // Check if there's a variant of this package installed
            if let Some(matched_id) = matcher.find_package(pkg_id, &installed_snapshot) {
                // If matched name is different from config name, it's a variant
                if matched_id.name != pkg_id.name {
                    // Check if this variant is NOT already tracked in state
                    let state_key = resolver::make_state_key(pkg_id);
                    let state_pkg = state.packages.get(&state_key);

                    // Only report if not tracked (means user might have manually changed it)
                    if state_pkg.is_none() || state_pkg.and_then(|s| s.aur_package_name.as_ref()).map(|n| n != &matched_id.name).unwrap_or(false) {
                        variant_mismatches.push((pkg_id.name.clone(), matched_id.name));
                    }
                }
            }
        }
    }

    // If variant mismatches found, error with helpful message
    if !variant_mismatches.is_empty() && !options.force {
        output::separator();
        output::error("Variant transition detected!");
        println!("\nThe following packages have different variants installed:\n");

        for (config_name, installed_name) in &variant_mismatches {
            println!("  {}  →  {}",
                config_name.cyan().bold(),
                installed_name.yellow().bold()
            );
        }

        println!("\n{}", "This requires explicit transition to avoid unintended changes.".dimmed());
        println!("\n{}", "To resolve this:".bold());
        println!("  1. For each package, run:");
        for (config_name, installed_name) in &variant_mismatches {
            println!("     {}",
                format!("declarch switch {} {}",
                    installed_name.yellow(),
                    config_name.cyan()
                ).bold()
            );
        }
        println!("\n  2. Or, update your config to match the installed variant");
        println!("\n  3. Use {} to bypass this check (not recommended)",
            "--force".yellow().bold());

        return Err(DeclarchError::Other(
            "Variant transition required. Use 'declarch switch' or update your config.".to_string()
        ));
    }

    // --- WARNINGS ---
    if !options.update && !tx.to_install.is_empty() {
        let should_warn = match state.meta.last_update {
            Some(last) => Utc::now().signed_duration_since(last).num_hours() > 24,
            None => true,
        };

        if should_warn {
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
    let allow_prune = matches!(sync_target, SyncTarget::All) || options.force;
    let should_prune = options.prune && allow_prune;

    display_transaction_plan(&tx, &installed_snapshot, should_prune);

    if tx.to_install.is_empty()
        && tx.to_adopt.is_empty()
        && tx.to_update_meta.is_empty()
        && (!should_prune || tx.to_prune.is_empty())
    {
        output::success("System is in sync.");
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
        if !output::prompt_yes_no("Proceed?") {
            return Err(DeclarchError::Interrupted);
        }
    }

    // -- INSTALLATION --
    execute_installations(&tx, &managers, &mut installed_snapshot)?;

    // -- REMOVAL --
    if should_prune {
        // A. BUILD PROTECTED LIST (Antidote for Aliasing Bug - FIXED)
        // Kumpulkan semua "Nama Asli" dari paket yang ADA DI CONFIG.
        // Tidak peduli apakah dia baru (Adopt) atau lama (Stable), fisiknya harus dilindungi.
        let mut protected_physical_names: Vec<String> = Vec::new();
        
        // ITERASI CONFIG LANGSUNG (Bukan Transaction)
        for pkg in config.packages.keys() {
            // Skip jika user mengecualikan paket ini
            if config.excludes.contains(&pkg.name) {
                continue;
            }

            let mut real_name = pkg.name.clone();

            // Try Exact Match
            if installed_snapshot.contains_key(pkg) {
                real_name = pkg.name.clone();
            } else {
                // Try Smart Match Resolution
                match pkg.backend {
                    Backend::Aur => {
                        let suffixes = ["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];
                        for suffix in suffixes {
                            let alt_name = format!("{}{}", pkg.name, suffix);
                            let alt_id = PackageId { name: alt_name.clone(), backend: Backend::Aur };
                            if installed_snapshot.contains_key(&alt_id) {
                                real_name = alt_name;
                                break;
                            }
                        }
                         // Prefix
                        if real_name == pkg.name {
                             if let Some((prefix, _)) = pkg.name.split_once('-') {
                                let alt_id = PackageId { name: prefix.to_string(), backend: Backend::Aur };
                                if installed_snapshot.contains_key(&alt_id) {
                                    real_name = prefix.to_string();
                                }
                            }
                        }
                    },
                    Backend::Flatpak => {
                        let search = pkg.name.to_lowercase();
                        for (installed_id, _) in &installed_snapshot {
                            if installed_id.backend == Backend::Flatpak {
                                if installed_id.name.to_lowercase().contains(&search) {
                                    real_name = installed_id.name.clone();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            protected_physical_names.push(real_name);
        }

        // B. EXECUTE REMOVAL
        let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
        
        for pkg in tx.to_prune.iter() {
            // ... (kode di bawah ini SAMA PERSIS dengan sebelumnya) ...
            // ... (Copy paste dari file lamamu mulai dari sini ke bawah) ...
            
            // 1. GHOST MODE (Static Check)
            if CRITICAL_PACKAGES.contains(&pkg.name.as_str()) {
                continue;
            }

            // 2. REAL ID RESOLUTION
            let mut real_name = pkg.name.clone();

            match pkg.backend {
                 Backend::Flatpak => {
                    let search = pkg.name.to_lowercase();
                    for (installed_id, _) in &installed_snapshot {
                        if installed_id.backend == Backend::Flatpak {
                            if installed_id.name.to_lowercase().contains(&search) {
                                real_name = installed_id.name.clone();
                                break; 
                            }
                        }
                    }
                },
                Backend::Aur => {
                    let suffixes = ["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];
                    let mut found = false;
                    for suffix in suffixes {
                        let alt_name = format!("{}{}", pkg.name, suffix);
                        let alt_id = PackageId { name: alt_name.clone(), backend: Backend::Aur };
                        if installed_snapshot.contains_key(&alt_id) {
                            real_name = alt_name;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        if let Some((prefix, _)) = pkg.name.split_once('-') {
                            let alt_id = PackageId { name: prefix.to_string(), backend: Backend::Aur };
                            if installed_snapshot.contains_key(&alt_id) {
                                real_name = prefix.to_string();
                            }
                        }
                    }
                }
            }

            // 3. FRATRICIDE CHECK (Dynamic Runtime Check)
            if protected_physical_names.contains(&real_name) {
                // Ini yang akan menyelamatkan hyprland kamu!
                println!("  ℹ Keeping physical package '{}' (claimed by active config)", real_name.dimmed());
                continue;
            }
            
            removes.entry(pkg.backend.clone()).or_default().push(real_name);
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
    update_state_after_sync(&mut state, &tx, &installed_snapshot, &options)?;

    output::success("Sync complete!");

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS - Break down the monolithic sync function
// ============================================================================

/// Display the transaction plan to the user
fn display_transaction_plan(
    tx: &resolver::Transaction,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    should_prune: bool,
) {
    output::separator();

    if !tx.to_install.is_empty() {
        println!("{}", "To Install:".green().bold());
        for pkg in &tx.to_install {
            println!("  + {}", pkg);
        }
    }

    if !tx.to_update_meta.is_empty() {
        println!("{}", "State Updates (Drift detected):".blue().bold());
        for pkg in &tx.to_update_meta {
            let v_display = installed_snapshot
                .get(pkg)
                .map(|m| m.version.as_deref().unwrap_or("?"))
                .unwrap_or("smart-match");
            println!("  ~ {} (v{})", pkg, v_display);
        }
    }

    if !tx.to_adopt.is_empty() {
        println!("{}", "To Adopt (Track in State):".yellow().bold());
        for pkg in &tx.to_adopt {
            println!("  ~ {}", pkg);
        }
    }

    if !tx.to_prune.is_empty() {
        let header = if should_prune {
            "To Remove:".red().bold()
        } else {
            "To Remove (Skipped):".dimmed()
        };
        println!("{}", header);
        for pkg in &tx.to_prune {
            let is_critical = CRITICAL_PACKAGES.contains(&pkg.name.as_str());
            if should_prune {
                if is_critical {
                    println!(
                        "  - {} {}",
                        pkg.to_string().yellow(),
                        "(Protected - Detaching from State)".italic()
                    );
                } else {
                    println!("  - {}", pkg);
                }
            } else {
                println!("  - {}", pkg.to_string().dimmed());
            }
        }
    }
}

/// Execute package installations
fn execute_installations(
    tx: &resolver::Transaction,
    managers: &HashMap<Backend, Box<dyn PackageManager>>,
    installed_snapshot: &mut HashMap<PackageId, PackageMetadata>,
) -> Result<()> {
    // Group packages by backend
    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install.iter() {
        installs
            .entry(pkg.backend.clone())
            .or_default()
            .push(pkg.name.clone());
    }

    // Install packages
    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend.to_string()));
            mgr.install(&pkgs)?;
        }
    }

    // Refresh snapshot after installations
    if !tx.to_install.is_empty() {
        output::info("Refreshing package state...");
        for (backend, mgr) in managers {
            if !mgr.is_available() {
                continue;
            }
            let packages = mgr.list_installed()?;
            for (name, meta) in packages {
                let id = PackageId {
                    name,
                    backend: backend.clone(),
                };
                installed_snapshot.insert(id, meta);
            }
        }
    }

    Ok(())
}

/// Find metadata for a package using smart matching
fn find_package_metadata<'a>(
    pkg: &PackageId,
    installed_snapshot: &'a HashMap<PackageId, PackageMetadata>,
) -> Option<&'a PackageMetadata> {
    // Try exact match first
    if let Some(meta) = installed_snapshot.get(pkg) {
        return Some(meta);
    }

    // Use PackageMatcher for smart matching
    let matcher = crate::core::matcher::PackageMatcher::new();
    let matched_id = matcher.find_package(pkg, installed_snapshot)?;
    installed_snapshot.get(&matched_id)
}

/// Update state file after sync
fn update_state_after_sync(
    state: &mut State,
    tx: &resolver::Transaction,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    options: &SyncOptions,
) -> Result<()> {
    // Collect all packages to upsert
    let mut to_upsert = tx.to_install.clone();
    to_upsert.extend(tx.to_adopt.clone());
    to_upsert.extend(tx.to_update_meta.clone());

    // Upsert packages into state
    for pkg in to_upsert {
        let meta = find_package_metadata(&pkg, installed_snapshot);
        let version = meta.and_then(|m| m.version.clone());
        let key = resolver::make_state_key(&pkg);

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                aur_package_name: None, // TODO: Discover actual AUR package name
                installed_at: Utc::now(),
                version,
            },
        );
    }

    // Prune from state if needed
    if options.prune {
        for pkg in &tx.to_prune {
            let key = resolver::make_state_key(&pkg);
            state.packages.remove(&key);
        }
    }

    // Update metadata
    state.meta.last_sync = Utc::now();
    if options.update {
        state.meta.last_update = Some(Utc::now());
    }

    state::io::save_state(&state)?;

    Ok(())
}
