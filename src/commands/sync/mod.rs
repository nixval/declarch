//! Sync command orchestration
//!
//! This module orchestrates the sync workflow by coordinating:
//! - Transaction planning (planner.rs)
//! - Package installation/removal (executor.rs)
//! - State updates (state_sync.rs)
//! - Hook execution (hooks.rs)
//! - Variant matching (variants.rs)

mod planner;
mod executor;
mod state_sync;
mod hooks;
mod variants;

// Re-export public API
pub use planner::{create_transaction, check_variant_transitions, warn_partial_upgrade, display_transaction_plan};
pub use executor::execute_transaction;
pub use state_sync::update_state;
pub use hooks::execute_sync_hooks;

use crate::config::loader;
use crate::core::types::SyncTarget;
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::distro::DistroType;
use crate::utils::install;
use crate::utils::paths;
use std::path::Path;

// Import remaining dependencies from original sync.rs
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{
    resolver,
    types::{PackageId, PackageMetadata},
};
use crate::packages::{PackageManager, create_manager};
use crate::state::{
    self,
    types::{Backend, State},
};
use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

// Type aliases to reduce complexity
pub type InstalledSnapshot = HashMap<PackageId, PackageMetadata>;
pub type ManagerMap = HashMap<Backend, Box<dyn PackageManager>>;

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
    // 1. Target Resolution
    let sync_target = resolve_target(&options.target);

    // 2. Load Config
    let config_path = paths::config_file()?;
    let mut config = if !options.modules.is_empty() {
        // Check if this is selective sync (single module, no target specified)
        // or loading additional modules
        if options.modules.len() == 1 && options.target.is_none() {
            // Selective sync: load ONLY this module (no verbose message)
            load_single_module(&config_path, &options.modules[0])?
        } else {
            // Load additional modules
            load_config_with_modules(&config_path, &options.modules)?
        }
    } else {
        loader::load_root_config(&config_path)?
    };

    // Execute pre-sync hooks
    execute_sync_hooks(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    // 3. System Update
    perform_system_update(&options)?;

    // 4. Initialize Managers & Snapshot
    let (mut installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 5. Load State & Resolve
    let mut state = state::io::load_state()?;

    // ==========================================
    // PHASE 2: VARIANT DETECTION
    // ==========================================
    let variant_updates = variants::detect_and_resolve_variant_transitions(
        &mut state,
        &installed_snapshot,
        &sync_target,
    )?;

    // ==========================================
    // PHASE 3: TRANSACTION PLANNING
    // ==========================================
    let tx = create_transaction(
        &mut config,
        &state,
        &installed_snapshot,
        &managers,
        &sync_target,
    )?;

    // ==========================================
    // PHASE 4: EXECUTE TRANSACTION
    // ==========================================
    if tx.to_install.is_empty()
        && tx.to_adopt.is_empty()
        && tx.to_update_project_metadata.is_empty()
        && (!options.prune || tx.to_prune.is_empty())
    {
        output::success("System is in sync.");
        if options.update {
            state.meta.last_update = Some(Utc::now());
            state::io::save_state_locked(&state)?;
        }
        // Execute post-sync hooks even when system is in sync
        execute_sync_hooks(
            &config.lifecycle_actions,
            options.hooks,
            options.dry_run,
        )?;
        return Ok(());
    }

    if options.dry_run {
        output::info("Dry run mode - no changes will be made");
        return Ok(());
    }

    execute_transaction(&tx, &managers, &config, &options)?;

    // ==========================================
    // PHASE 5: UPDATE STATE
    // ==========================================
    let state = update_state(&state, &tx, &installed_snapshot, &options)?;

    // ==========================================
    // PHASE 6: POST-SYNC HOOKS
    // ==========================================
    execute_sync_hooks(
        &config.lifecycle_actions,
        options.hooks,
        options.dry_run,
    )?;

    Ok(())
}

/// Try to find an AUR package variant in the installed snapshot
/// Returns the variant name if found, otherwise None
fn find_aur_variant(
    package_name: &str,
    installed_snapshot: &InstalledSnapshot,
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
    installed_snapshot: &InstalledSnapshot,
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
/// Load a single module file (for selective sync)
/// Loads ONLY the specified module, not the entire config
fn load_single_module(_config_path: &Path, module_name: &str) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

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

    // Load ONLY this module's packages (not the full config)
    let module_config = loader::load_root_config(&final_path)?;

    Ok(module_config)
}

fn load_config_with_modules(
    config_path: &Path,
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
