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
pub use state_sync::{update_state, update_state_with_success};
pub use hooks::{execute_on_failure, execute_on_success, execute_post_sync, execute_pre_sync};
pub use variants::{find_variant, resolve_installed_package_name};

use crate::config::loader;
use crate::core::types::SyncTarget;
use crate::error::Result;
use crate::ui as output;
use crate::utils::paths;
use std::path::Path;

use crate::core::types::{PackageId, PackageMetadata};
use crate::packages::{PackageManager, create_manager};
use crate::state::types::Backend;
use crate::state;
use std::collections::HashMap;

// Re-export dry-run display function
pub use planner::display_dry_run_details;

// Type aliases to reduce complexity
pub type InstalledSnapshot = HashMap<PackageId, PackageMetadata>;
pub type ManagerMap = HashMap<Backend, Box<dyn PackageManager>>;

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
    pub modules: Vec<String>,
    pub diff: bool,
}

pub fn run(options: SyncOptions) -> Result<()> {
    // Acquire exclusive lock at the very beginning to prevent concurrent sync
    // Lock is held until this function returns (RAII pattern)
    let lock = if options.dry_run {
        // Dry-run doesn't need exclusive lock, but we check if another process is running
        match state::io::acquire_lock() {
            Ok(lock) => Some(lock),
            Err(_) => {
                output::warning("Another declarch process is running. Dry-run may show stale state.");
                None
            }
        }
    } else {
        // Real sync requires exclusive lock
        Some(state::io::acquire_lock().map_err(|e| {
            crate::error::DeclarchError::Other(format!(
                "Cannot start sync: {}\n\
                 If no other declarch process is running, delete the lock file manually.",
                e
            ))
        })?)
    };

    // 1. Load Config
    let config_path = paths::config_file()?;
    let mut config = if !options.modules.is_empty() {
        if options.modules.len() == 1 && options.target.is_none() {
            load_single_module(&config_path, &options.modules[0])?
        } else {
            load_config_with_modules(&config_path, &options.modules)?
        }
    } else {
        loader::load_root_config(&config_path)?
    };

    // 2. Target Resolution
    let sync_target = resolve_target(&options.target, &config);

    // Execute pre-sync hooks
    execute_pre_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    // 3. Initialize Managers & Snapshot
    let (installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 3.5. Run backend updates if --update flag is set
    if options.update && !options.dry_run {
        execute_backend_updates(&managers)?;
    }

    // 4. Load State & Resolve
    let state = state::io::load_state()?;

    // 5. Create Transaction
    let transaction = create_transaction(&mut config, &state, &installed_snapshot, &managers, &sync_target)?;

    // 5.5 Check for dangerous variant transitions and warn about stale updates
    check_variant_transitions(
        &config,
        &installed_snapshot,
        &state,
        &transaction,
        &sync_target,
        &options,
    )?;
    warn_partial_upgrade(&state, &transaction, &options);

    // 6. Display Plan
    if transaction.to_install.is_empty() 
        && transaction.to_prune.is_empty() 
        && transaction.to_adopt.is_empty() {
        output::success("Everything is up to date!");
        execute_post_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;
        execute_on_success(&config.lifecycle_actions, options.hooks, options.dry_run)?;
        return Ok(());
    }

    // Show detailed dry-run info or regular plan
    if options.dry_run {
        display_dry_run_details(&transaction, options.prune, &installed_snapshot);
    } else {
        display_transaction_plan(&transaction, options.prune);
    }

    // Handle --diff flag: Show diff and exit (like git diff)
    if options.diff {
        show_sync_diff(&transaction, &installed_snapshot);
        return Ok(());
    }

    // 7. Execute
    if !options.dry_run {
        if !options.yes && !output::prompt_yes_no("Proceed with sync?") {
            output::info("Sync cancelled");
            return Err(crate::error::DeclarchError::Interrupted);
        }

        let successfully_installed = match execute_transaction(&transaction, &managers, &config, &options) {
            Ok(installed) => installed,
            Err(e) => {
                let _ = execute_on_failure(&config.lifecycle_actions, options.hooks, options.dry_run);
                return Err(e);
            }
        };

        // 8. Refresh installed snapshot and update state with successful packages
        let post_execution_snapshot = refresh_installed_snapshot(&managers);

        let new_state = update_state_with_success(
            &state,
            &transaction,
            &post_execution_snapshot,
            &options,
            &successfully_installed,
        )?;
        
        // Save state with lock held (ensures no concurrent modifications)
        if let Some(ref lock) = lock {
            if let Err(e) = state::io::save_state_locked(&new_state, lock) {
                let _ = execute_on_failure(&config.lifecycle_actions, options.hooks, options.dry_run);
                return Err(e);
            }
        } else {
            // This shouldn't happen for non-dry-run, but handle gracefully
            if let Err(e) = state::io::save_state(&new_state) {
                let _ = execute_on_failure(&config.lifecycle_actions, options.hooks, options.dry_run);
                return Err(e);
            }
        }
    } else {
        // Dry-run complete
        output::success("Dry-run completed - no changes were made");
    }

    // Execute post-sync hooks
    execute_post_sync(&config.lifecycle_actions, options.hooks, options.dry_run)?;
    execute_on_success(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    Ok(())
}

/// Show diff view of sync changes
fn show_sync_diff(
    transaction: &crate::core::resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
) {
    use colored::Colorize;

    output::header("Sync Diff");

    // Show packages to install
    if !transaction.to_install.is_empty() {
        println!("\n{}:", "Packages to install".green().bold());
        for pkg_id in &transaction.to_install {
            println!("  {} {} {}", "+".green(), pkg_id.backend, pkg_id.name);
        }
    }

    // Show packages to remove
    if !transaction.to_prune.is_empty() {
        println!("\n{}:", "Packages to remove".red().bold());
        for pkg_id in &transaction.to_prune {
            let version = installed_snapshot
                .get(pkg_id)
                .and_then(|m| m.version.as_ref())
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            println!("  {} {} {}{}", "-".red(), pkg_id.backend, pkg_id.name, version.dimmed());
        }
    }

    // Show packages to adopt
    if !transaction.to_adopt.is_empty() {
        println!("\n{}:", "Packages to adopt".yellow().bold());
        for pkg_id in &transaction.to_adopt {
            println!("  {} {} {}", "~".yellow(), pkg_id.backend, pkg_id.name);
        }
    }

    // Summary
    println!();
    let total_changes = transaction.to_install.len() + transaction.to_prune.len() + transaction.to_adopt.len();
    output::info(&format!("Total changes: {}", total_changes));
    output::info("Run 'declarch sync' to apply these changes");
}

fn resolve_target(target: &Option<String>, config: &loader::MergedConfig) -> SyncTarget {
    if let Some(t) = target {
        let normalized_backend = Backend::from(t.as_str());
        let matches_backend_in_packages = config
            .packages
            .keys()
            .any(|pkg_id| pkg_id.backend == normalized_backend);
        let matches_backend_in_imports = config
            .backends
            .iter()
            .any(|backend| backend.name.eq_ignore_ascii_case(t));

        if matches_backend_in_packages || matches_backend_in_imports {
            SyncTarget::Backend(normalized_backend)
        } else {
            SyncTarget::Named(t.clone())
        }
    } else {
        SyncTarget::All
    }
}

fn initialize_managers_and_snapshot(
    config: &loader::MergedConfig,
    options: &SyncOptions,
    sync_target: &SyncTarget,
) -> Result<(InstalledSnapshot, ManagerMap)> {
    let mut installed_snapshot: InstalledSnapshot = HashMap::new();
    let mut managers: ManagerMap = HashMap::new();

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
                let available = manager.is_available();

                if !available && matches!(sync_target, SyncTarget::Backend(b) if b == &backend) {
                    output::warning(&format!(
                        "Backend '{}' is not available on this system.",
                        backend
                    ));
                }

                if available {
                    match manager.list_installed() {
                        Ok(packages) => {
                            for (name, meta) in packages {
                                let pkg_id = PackageId {
                                    name: name.clone(),
                                    backend: backend.clone(),
                                };
                                installed_snapshot.insert(pkg_id, meta);
                            }
                        }
                        Err(e) => {
                            output::warning(&format!(
                                "Failed to list packages for {}: {}",
                                backend, e
                            ));
                        }
                    }
                    managers.insert(backend.clone(), manager);
                }
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to create manager for {}: {}",
                    backend, e
                ));
            }
        }
    }

    Ok((installed_snapshot, managers))
}

fn refresh_installed_snapshot(managers: &ManagerMap) -> InstalledSnapshot {
    let mut snapshot = InstalledSnapshot::new();
    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }
        match manager.list_installed() {
            Ok(packages) => {
                for (name, meta) in packages {
                    snapshot.insert(
                        PackageId {
                            name,
                            backend: backend.clone(),
                        },
                        meta,
                    );
                }
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to refresh package snapshot for {}: {}",
                    backend, e
                ));
            }
        }
    }
    snapshot
}

/// Execute update for all backends that support it
fn execute_backend_updates(managers: &ManagerMap) -> Result<()> {
    output::separator();
    output::info("Updating package indices...");

    let mut updated_count = 0;
    let mut skipped_count = 0;

    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }

        if !manager.supports_update() {
            output::info(&format!(
                "Skipping '{}': no update_cmd configured",
                backend
            ));
            skipped_count += 1;
            continue;
        }

        match manager.update() {
            Ok(()) => {
                updated_count += 1;
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to update '{}': {}",
                    backend, e
                ));
                skipped_count += 1;
            }
        }
    }

    if updated_count > 0 {
        output::info(&format!("Updated {} backend(s)", updated_count));
    }
    if skipped_count > 0 {
        output::info(&format!("Skipped {} backend(s)", skipped_count));
    }

    Ok(())
}

fn load_single_module(_config_path: &Path, module_name: &str) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    let module_path = paths::module_file(module_name);

    let final_path = if let Ok(path) = module_path {
        if path.exists() {
            path
        } else {
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

    let module_config = loader::load_root_config(&final_path)?;
    Ok(module_config)
}

fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    let mut merged = loader::load_root_config(config_path)?;

    for module_name in extra_modules {
        let module_path = paths::module_file(module_name);

        let final_path = if let Ok(path) = module_path {
            if path.exists() {
                path
            } else {
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

        output::info(&format!("  Loading module: {}", final_path.display()));
        let module_config = loader::load_root_config(&final_path)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}
