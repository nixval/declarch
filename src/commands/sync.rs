use crate::utils::paths;
use crate::utils::distro::DistroType;
use crate::utils::install;
use crate::ui as output;
use crate::state::{self, types::{Backend, PackageState, State}};
use crate::config::loader;
use crate::core::{resolver, types::{PackageId, PackageMetadata, SyncTarget}};
use crate::packages::{PackageManager, create_manager};
use crate::error::{DeclarchError, Result};
use colored::Colorize;
use chrono::Utc;
use std::collections::HashMap;
use std::process::{Command, Stdio};

mod critical {
    pub const PACKAGES: &[&str] = &[
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
}

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
    pub skip_soar_install: bool,
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

    // Detect distro and create available backends
    let distro = DistroType::detect();
    let global_config = crate::config::types::GlobalConfig::default();

    // Get backends from config (unique set)
    let configured_backends: std::collections::HashSet<Backend> = config.packages.keys()
        .map(|pkg_id| pkg_id.backend.clone())
        .collect();

    // Initialize managers for configured backends
    for backend in configured_backends {
        match create_manager(&backend, &global_config, options.noconfirm) {
            Ok(manager) => {
                let mut available = manager.is_available();

                // Special handling for Soar: try to install if missing
                if matches!(backend, Backend::Soar) && !available && !options.skip_soar_install && !options.dry_run {
                    output::warning(&format!("Soar is required but not installed"));

                    // Try to install Soar
                    if install::install_soar()? {
                        output::success("Soar installed successfully!");
                        available = true;
                    } else {
                        output::warning("Skipping Soar packages - automatic installation failed");
                    }
                }

                // Warn if targeting unavailable backend
                if !available && matches!(sync_target, SyncTarget::Backend(ref b) if b == &backend) {
                    output::warning(&format!("Backend '{}' is not available on this system.", backend));
                }

                if available {
                    // List installed packages from this backend
                    match manager.list_installed() {
                        Ok(packages) => {
                            for (name, meta) in packages {
                                let id = PackageId { name, backend: backend.clone() };
                                installed_snapshot.insert(id, meta);
                            }
                            managers.insert(backend.clone(), manager);
                        },
                        Err(e) => {
                            output::warning(&format!("Failed to list packages from {}: {}", backend, e));
                        }
                    }
                }
            },
            Err(e) => {
                output::warning(&format!("Failed to initialize {} backend: {}", backend, e));
            }
        }
    }

    // On non-Arch systems, warn about AUR packages in config
    if !distro.supports_aur() {
        let has_aur_packages = config.packages.keys().any(|pkg_id| matches!(pkg_id.backend, Backend::Aur));
        if has_aur_packages {
            output::warning("AUR packages detected but system is not Arch-based. These will be skipped.");
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
        // Collect all actual installed package names from config to protect them
        let mut protected_physical_names: Vec<String> = Vec::new();

        // Iterate directly through config packages
        for pkg in config.packages.keys() {
            // Skip if user excluded this package
            if config.excludes.contains(&pkg.name) {
                continue;
            }

            let real_name = resolve_installed_package_name(pkg, &installed_snapshot);
            protected_physical_names.push(real_name);
        }

        // B. EXECUTE REMOVAL
        let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
        
        for pkg in tx.to_prune.iter() {
            // 1. GHOST MODE (Static Check)
            if critical::PACKAGES.contains(&pkg.name.as_str()) {
                continue;
            }

            // 2. REAL ID RESOLUTION (using helper function)
            let real_name = resolve_installed_package_name(pkg, &installed_snapshot);

            // 3. FRATRICIDE CHECK (Dynamic Runtime Check)
            if protected_physical_names.contains(&real_name) {
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

/// Smart matching: Find the actual installed package name for a config package
/// This handles variant matching (e.g., "hyprland" → "hyprland-git")
fn resolve_installed_package_name(
    pkg: &PackageId,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
) -> String {
    let mut real_name = pkg.name.clone();

    // Try exact match first
    if installed_snapshot.contains_key(pkg) {
        return real_name;
    }

    // Try smart match based on backend
    match pkg.backend {
        Backend::Aur => {
            let suffixes = ["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];
            for suffix in suffixes {
                let alt_name = format!("{}{}", pkg.name, suffix);
                let alt_id = PackageId { name: alt_name.clone(), backend: Backend::Aur };
                if installed_snapshot.contains_key(&alt_id) {
                    real_name = alt_name;
                    return real_name;
                }
            }
            // Try prefix match
            if let Some((prefix, _)) = pkg.name.split_once('-') {
                let alt_id = PackageId { name: prefix.to_string(), backend: Backend::Aur };
                if installed_snapshot.contains_key(&alt_id) {
                    real_name = prefix.to_string();
                }
            }
        },
        Backend::Flatpak => {
            let search = pkg.name.to_lowercase();
            for (installed_id, _) in installed_snapshot {
                if installed_id.backend == Backend::Flatpak {
                    if installed_id.name.to_lowercase().contains(&search) {
                        real_name = installed_id.name.clone();
                        return real_name;
                    }
                }
            }
        }
        Backend::Soar => {
            // Soar requires exact matching - no smart matching needed
            real_name = pkg.name.clone();
        }
    }

    real_name
}

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
            let is_critical = critical::PACKAGES.contains(&pkg.name.as_str());
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
