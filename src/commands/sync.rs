use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{
    resolver,
    types::{PackageId, PackageMetadata, SyncTarget},
};
use crate::error::{DeclarchError, Result};
use crate::packages::{PackageManager, create_manager};
use crate::state::{
    self,
    types::{Backend, PackageState, State},
};
use crate::ui as output;
use crate::utils::distro::DistroType;
use crate::utils::install;
use crate::utils::paths;
use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

// Type aliases to reduce complexity
type InstalledSnapshot = HashMap<PackageId, PackageMetadata>;
type ManagerMap = HashMap<Backend, Box<dyn PackageManager>>;

/// AUR package variant suffixes for smart matching
const AUR_SUFFIXES: &[&str] = &["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];

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
    pub hooks: bool,
    pub skip_soar_install: bool,
    pub modules: Vec<String>,
}

pub fn run(options: SyncOptions) -> Result<()> {
    output::header("Synchronizing Packages");

    // 1. Target Resolution
    let sync_target = resolve_target(&options.target);

    // 2. Load Config
    let config_path = paths::config_file()?;
    let mut config = if !options.modules.is_empty() {
        output::info(&format!("Loading additional modules: {:?}", options.modules));
        load_config_with_modules(&config_path, &options.modules)?
    } else {
        loader::load_root_config(&config_path)?
    };

    // Execute pre-sync hooks
    crate::commands::hooks::execute_pre_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    // 3. System Update
    perform_system_update(&options)?;

    // 4. Initialize Managers & Snapshot
    let (mut installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 5. Load State & Resolve
    let mut state = state::io::load_state()?;

    // Filter packages by available backends and resolve transaction
    let tx = resolve_and_filter_packages(
        &mut config,
        &state,
        &installed_snapshot,
        &managers,
        &sync_target
    )?;

    // Check for variant mismatches
    check_variant_transitions(&config, &installed_snapshot, &state, &tx, &sync_target, &options)?;

    // Warn about partial upgrades
    warn_partial_upgrade(&state, &tx, &options);

    // 6. Display Plan
    let allow_prune = matches!(sync_target, SyncTarget::All) || options.force;
    let should_prune = options.prune && allow_prune;

    display_transaction_plan(&tx, &installed_snapshot, should_prune);

    if tx.to_install.is_empty()
        && tx.to_adopt.is_empty()
        && tx.to_update_project_metadata.is_empty()
        && (!should_prune || tx.to_prune.is_empty())
    {
        output::success("System is in sync.");
        if options.update {
            state.meta.last_update = Some(Utc::now());
            state::io::save_state(&state)?;
        }
        // Execute post-sync hooks even when system is in sync
        crate::commands::hooks::execute_post_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;
        return Ok(());
    }

    if options.dry_run {
        return Ok(());
    }

    // 7. Execution
    let skip_prompt = options.yes || options.noconfirm || options.force;
    if !skip_prompt && !output::prompt_yes_no("Proceed?") {
        return Err(DeclarchError::Interrupted);
    }

    // -- INSTALLATION --
    execute_installations(&tx, &managers, &mut installed_snapshot)?;

    // -- REMOVAL --
    if should_prune {
        execute_pruning(&config, &tx, &managers, &installed_snapshot)?;
    }

    // 8. Update State
    update_state_after_sync(&mut state, &tx, &installed_snapshot, &options)?;

    // Execute post-sync hooks
    crate::commands::hooks::execute_post_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    output::success("Sync complete!");

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS - Break down the monolithic sync function
// ============================================================================

/// Try to find an AUR package variant in the installed snapshot
/// Returns the variant name if found, otherwise None
fn find_aur_variant(
    package_name: &str,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
) -> Option<String> {
    // Try each suffix variant
    for suffix in AUR_SUFFIXES {
        let alt_name = format!("{}{}", package_name, suffix);
        let alt_id = PackageId {
            name: alt_name.clone(),
            backend: Backend::Aur,
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(alt_name);
        }
    }

    // Try prefix match (e.g., "hyprland-git" → "hyprland")
    if let Some((prefix, _)) = package_name.split_once('-') {
        let alt_id = PackageId {
            name: prefix.to_string(),
            backend: Backend::Aur,
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(prefix.to_string());
        }
    }

    None
}

/// Smart matching: Find the actual installed package name for a config package
/// This handles variant matching (e.g., "hyprland" → "hyprland-git")
fn resolve_installed_package_name(
    pkg: &PackageId,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
) -> String {
    // Try exact match first
    if installed_snapshot.contains_key(pkg) {
        return pkg.name.clone();
    }

    // Try smart match based on backend
    match pkg.backend {
        Backend::Aur => {
            // Use helper function for variant matching
            if let Some(variant) = find_aur_variant(&pkg.name, installed_snapshot) {
                return variant;
            }
            pkg.name.clone()
        }
        Backend::Flatpak => {
            let search = pkg.name.to_lowercase();
            for installed_id in installed_snapshot.keys() {
                if installed_id.backend == Backend::Flatpak
                    && installed_id.name.to_lowercase().contains(&search)
                {
                    return installed_id.name.clone();
                }
            }
            pkg.name.clone()
        }
        Backend::Soar
        | Backend::Npm
        | Backend::Yarn
        | Backend::Pnpm
        | Backend::Bun
        | Backend::Pip
        | Backend::Cargo
        | Backend::Brew
        | Backend::Custom(_) => {
            // These backends require exact matching - no smart matching needed
            pkg.name.clone()
        }
    }
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

    if !tx.to_update_project_metadata.is_empty() {
        println!("{}", "State Updates (Drift detected):".blue().bold());
        for pkg in &tx.to_update_project_metadata {
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
            output::info(&format!("Installing {} packages...", backend));
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

/// Discover the actual AUR package name for a given package
/// This handles cases where the config name differs from the actual AUR package name
/// (e.g., config says "hyprland" but AUR package is "hyprland-git")
fn discover_aur_package_name(package_name: &str) -> Option<String> {
    // Query pacman -Qi to get package info
    let output = Command::new("pacman")
        .args(["-Qi", package_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;

    // Parse the "Name" field to get the actual package name
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name") {
            if let Some(name) = line.split(':').nth(1) {
                let actual_name = name.trim();
                // Return the actual name only if it differs from config name
                if actual_name != package_name {
                    return Some(actual_name.to_string());
                }
            }
            break;
        }
    }

    None
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
    // Collect all packages to upsert (avoid cloning entire vectors)
    // Use iterators to chain the package collections
    let packages_to_upsert = tx.to_install
        .iter()
        .chain(tx.to_adopt.iter())
        .chain(tx.to_update_project_metadata.iter());

    // Upsert packages into state
    for pkg in packages_to_upsert {
        let meta = find_package_metadata(pkg, installed_snapshot);
        let version = meta.and_then(|m| m.version.clone());
        let key = resolver::make_state_key(pkg);

        // Discover actual AUR package name if applicable
        let aur_package_name = if pkg.backend == Backend::Aur {
            discover_aur_package_name(&pkg.name)
        } else {
            None
        };

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                aur_package_name,
                installed_at: Utc::now(),
                version,
            },
        );
    }

    // Prune from state if needed
    if options.prune {
        for pkg in &tx.to_prune {
            let key = resolver::make_state_key(pkg);
            state.packages.remove(&key);
        }
    }

    // Update metadata
    state.meta.last_sync = Utc::now();
    if options.update {
        state.meta.last_update = Some(Utc::now());
    }

    state::io::save_state(state)?;

    Ok(())
}

fn resolve_target(target: &Option<String>) -> SyncTarget {
    if let Some(t) = target {
        match t.to_lowercase().as_str() {
            "aur" | "repo" | "paru" | "pacman" => SyncTarget::Backend(Backend::Aur),
            "flatpak" => SyncTarget::Backend(Backend::Flatpak),
            _ => SyncTarget::Named(t.clone()),
        }
    } else {
        SyncTarget::All
    }
}

fn perform_system_update(options: &SyncOptions) -> Result<()> {
    let global_config = crate::config::types::GlobalConfig::default();
    let aur_helper = global_config.aur_helper.to_string();

    if options.update {
        output::info("Updating system...");
        if !options.dry_run {
            let mut cmd = Command::new(&aur_helper);
            cmd.arg("-Syu");
            if options.yes || options.noconfirm {
                cmd.arg("--noconfirm");
            }

            let status = cmd
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .status()?;
            if !status.success() {
                return Err(DeclarchError::Other("System update failed".into()));
            }
        }
    }
    Ok(())
}

fn initialize_managers_and_snapshot(
    config: &crate::config::loader::MergedConfig,
    options: &SyncOptions,
    sync_target: &SyncTarget,
) -> Result<(InstalledSnapshot, ManagerMap)> {
    output::info("Scanning system state...");
    let mut installed_snapshot: InstalledSnapshot = HashMap::new();
    let mut managers: ManagerMap = HashMap::new();

    // Detect distro and create available backends
    let distro = DistroType::detect();
    let global_config = crate::config::types::GlobalConfig::default();

    // Get backends from config (unique set)
    let configured_backends: std::collections::HashSet<Backend> = config
        .packages
        .keys()
        .map(|pkg_id| pkg_id.backend.clone())
        .collect();

    // Initialize managers for configured backends
    for backend in configured_backends {
        match create_manager(&backend, &global_config, options.noconfirm) {
            Ok(manager) => {
                let mut available = manager.is_available();

                // Special handling for Soar: try to install if missing
                if matches!(backend, Backend::Soar)
                    && !available
                    && !options.skip_soar_install
                    && !options.dry_run
                {
                    output::warning("Soar is required but not installed");

                    // Try to install Soar
                    if install::install_soar()? {
                        output::success("Soar installed successfully!");
                        available = true;
                    } else {
                        output::warning("Skipping Soar packages - automatic installation failed");
                    }
                }

                // Warn if targeting unavailable backend
                if !available && matches!(sync_target, SyncTarget::Backend(b) if b == &backend) {
                    output::warning(&format!(
                        "Backend '{}' is not available on this system.",
                        backend
                    ));
                }

                if available {
                    // List installed packages from this backend
                    match manager.list_installed() {
                        Ok(packages) => {
                            for (name, meta) in packages {
                                let id = PackageId {
                                    name,
                                    backend: backend.clone(),
                                };
                                installed_snapshot.insert(id, meta);
                            }
                            managers.insert(backend.clone(), manager);
                        }
                        Err(e) => {
                            output::warning(&format!(
                                "Failed to list packages from {}: {}",
                                backend, e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                output::warning(&format!("Failed to initialize {} backend: {}", backend, e));
            }
        }
    }

    // On non-Arch systems, warn about AUR packages in config
    if !distro.supports_aur() {
        let has_aur_packages = config
            .packages
            .keys()
            .any(|pkg_id| matches!(pkg_id.backend, Backend::Aur));
        if has_aur_packages {
            output::warning(
                "AUR packages detected but system is not Arch-based. These will be skipped.",
            );
        }
    }

    Ok((installed_snapshot, managers))
}

/// Resolve transaction and filter packages by available backends
fn resolve_and_filter_packages(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    // Filter packages to only include available backends
    let available_backends: std::collections::HashSet<Backend> = managers.keys().cloned().collect();
    let total_packages = config.packages.len();

    // Create filtered packages map
    let filtered_packages: std::collections::HashMap<_, _> = config
        .packages
        .iter()
        .filter(|(pkg_id, _)| available_backends.contains(&pkg_id.backend))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // Warn about packages from unavailable backends
    let skipped_count = total_packages - filtered_packages.len();
    if skipped_count > 0 {
        output::warning(&format!(
            "Skipping {} package(s) from unavailable backends.",
            skipped_count
        ));

        // Show which packages were skipped
        for (pkg_id, _) in config.packages.iter() {
            if !available_backends.contains(&pkg_id.backend) {
                output::info(&format!(
                    "  Skipping {} (backend '{}' not available)",
                    pkg_id.name, pkg_id.backend
                ));
            }
        }
    }

    // Temporarily replace packages in config with filtered version for resolver
    let original_packages = std::mem::replace(&mut config.packages, filtered_packages);

    let tx = resolver::resolve(config, state, installed_snapshot, sync_target)?;

    // Restore original packages after resolve
    config.packages = original_packages;

    Ok(tx)
}

/// Check for variant package mismatches between config and installed state
fn check_variant_transitions(
    config: &loader::MergedConfig,
    installed_snapshot: &InstalledSnapshot,
    state: &State,
    tx: &resolver::Transaction,
    sync_target: &SyncTarget,
    options: &SyncOptions,
) -> Result<()> {
    let matcher = crate::core::matcher::PackageMatcher::new();
    let mut variant_mismatches: Vec<(String, String)> = Vec::new();

    // Only check for variant transitions in full sync or when targeting specific backends
    if matches!(sync_target, SyncTarget::All | SyncTarget::Backend(_)) {
        // Re-filter packages for variant checking
        let available_backends: std::collections::HashSet<Backend> =
            installed_snapshot.keys().map(|pkg_id| pkg_id.backend.clone()).collect();

        for pkg_id in config
            .packages
            .keys()
            .filter(|pkg_id| available_backends.contains(&pkg_id.backend))
        {
            // Skip if this package is already in transaction to install
            if tx.to_install.iter().any(|p| p.name == pkg_id.name) {
                continue;
            }

            // Check if there's a variant of this package installed
            if let Some(matched_id) = matcher.find_package(pkg_id, installed_snapshot) {
                // If matched name is different from config name, it's a variant
                if matched_id.name != pkg_id.name {
                    // Check if this variant is NOT already tracked in state
                    let state_key = resolver::make_state_key(pkg_id);
                    let state_pkg = state.packages.get(&state_key);

                    // Only report if not tracked (means user might have manually changed it)
                    if state_pkg.is_none()
                        || state_pkg
                            .and_then(|s| s.aur_package_name.as_ref())
                            .map(|n| n != &matched_id.name)
                            .unwrap_or(false)
                    {
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
            println!(
                "  {}  →  {}",
                config_name.cyan().bold(),
                installed_name.yellow().bold()
            );
        }

        println!(
            "\n{}",
            "This requires explicit transition to avoid unintended changes.".dimmed()
        );
        println!("\n{}", "To resolve this:".bold());
        println!("  1. For each package, run:");
        for (config_name, installed_name) in &variant_mismatches {
            println!(
                "     {}",
                format!(
                    "declarch switch {} {}",
                    installed_name.yellow(),
                    config_name.cyan()
                )
                .bold()
            );
        }
        println!("\n  2. Or, update your config to match the installed variant");
        println!(
            "\n  3. Use {} to bypass this check (not recommended)",
            "--force".yellow().bold()
        );

        return Err(DeclarchError::Other(
            "Variant transition required. Use 'declarch switch' or update your config.".to_string(),
        ));
    }

    Ok(())
}

/// Warn about partial upgrades when system hasn't been updated recently
fn warn_partial_upgrade(state: &State, tx: &resolver::Transaction, options: &SyncOptions) {
    if !options.update && !tx.to_install.is_empty() {
        let should_warn = match state.meta.last_update {
            Some(last) => Utc::now().signed_duration_since(last).num_hours() > 24,
            None => true,
        };

        if should_warn {
            let time_str = state
                .meta
                .last_update
                .map(|t| format!("{}h ago", Utc::now().signed_duration_since(t).num_hours()))
                .unwrap_or("unknown".to_string());

            output::separator();
            println!(
                "{} Last system update: {}. Use {} to refresh.",
                "⚠ Partial Upgrade Risk:".yellow().bold(),
                time_str.white(),
                "--update".bold()
            );
        }
    }
}

/// Execute package pruning with safety checks
fn execute_pruning(
    config: &loader::MergedConfig,
    tx: &resolver::Transaction,
    managers: &ManagerMap,
    installed_snapshot: &InstalledSnapshot,
) -> Result<()> {
    // Build protected list - collect all actual installed package names from config
    let mut protected_physical_names: Vec<String> = Vec::new();

    for pkg in config.packages.keys() {
        // Skip if user excluded this package
        if config.excludes.contains(&pkg.name) {
            continue;
        }

        let real_name = resolve_installed_package_name(pkg, installed_snapshot);
        protected_physical_names.push(real_name);
    }

    // Build removal list
    let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();

    for pkg in tx.to_prune.iter() {
        // 1. GHOST MODE (Static Check) - Skip critical packages
        if CRITICAL_PACKAGES.contains(&pkg.name.as_str()) {
            continue;
        }

        // 2. REAL ID RESOLUTION (using helper function)
        let real_name = resolve_installed_package_name(pkg, installed_snapshot);

        // 3. FRATRICIDE CHECK (Dynamic Runtime Check) - Don't remove if protected
        if protected_physical_names.contains(&real_name) {
            println!(
                "  ℹ Keeping physical package '{}' (claimed by active config)",
                real_name.dimmed()
            );
            continue;
        }

        removes
            .entry(pkg.backend.clone())
            .or_default()
            .push(real_name);
    }

    // Execute removals
    for (backend, pkgs) in removes {
        if !pkgs.is_empty()
            && let Some(mgr) = managers.get(&backend)
        {
            output::info(&format!("Removing {} packages...", backend));
            mgr.remove(&pkgs)?;
        }
    }

    Ok(())
}

/// Load config with additional modules
fn load_config_with_modules(
    config_path: &std::path::PathBuf,
    extra_modules: &[String],
) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    // Load base config
    let mut merged = loader::load_root_config(config_path)?;

    // Load each additional module
    for module_name in extra_modules {
        // Try as module name (e.g., "gaming" -> modules/gaming.kdl)
        let module_path = paths::module_file(module_name);

        let final_path = if let Ok(path) = module_path {
            if path.exists() {
                path
            } else {
                // Try as direct path
                let direct_path = PathBuf::from(module_name);
                if direct_path.exists() {
                    direct_path
                } else {
                    return Err(crate::error::DeclarchError::Other(format!(
                        "Module not found: {}",
                        module_name
                    )));
                }
            }
        } else {
            // Try as direct path
            let direct_path = PathBuf::from(module_name);
            if direct_path.exists() {
                direct_path
            } else {
                return Err(crate::error::DeclarchError::Other(format!(
                    "Module not found: {}",
                    module_name
                )));
            }
        };

        // Load the module
        output::info(&format!("  Loading module: {}", final_path.display()));

        // Load the module config
        let module_config = loader::load_root_config(&final_path)?;

        // Merge the module config into our existing config
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}
