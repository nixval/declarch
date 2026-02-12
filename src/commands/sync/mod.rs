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
}

pub fn run(options: SyncOptions) -> Result<()> {
    // 1. Target Resolution
    let sync_target = resolve_target(&options.target);

    // 2. Load Config
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

    // Execute pre-sync hooks
    execute_sync_hooks(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    // 3. Initialize Managers & Snapshot
    let (installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 4. Load State & Resolve
    let state = state::io::load_state()?;

    // 5. Create Transaction
    let transaction = create_transaction(&mut config, &state, &installed_snapshot, &managers, &sync_target)?;

    // 6. Display Plan
    if transaction.to_install.is_empty() 
        && transaction.to_prune.is_empty() 
        && transaction.to_adopt.is_empty() {
        output::success("Everything is up to date!");
        return Ok(());
    }

    // Show detailed dry-run info or regular plan
    if options.dry_run {
        display_dry_run_details(&transaction, options.prune, &installed_snapshot);
    } else {
        display_transaction_plan(&transaction, options.prune);
    }

    // 7. Execute
    if !options.dry_run {
        if !options.yes && !output::prompt_yes_no("Proceed with sync?") {
            output::info("Sync cancelled");
            return Err(crate::error::DeclarchError::Interrupted);
        }

        execute_transaction(&transaction, &managers, &config, &options)?;

        // 8. Update State
        let new_state = update_state(&state, &transaction, &installed_snapshot, &options)?;
        state::io::save_state_locked(&new_state)?;
    } else {
        // Dry-run complete
        output::success("Dry-run completed - no changes were made");
    }

    // Execute post-sync hooks
    execute_sync_hooks(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    Ok(())
}

fn resolve_target(target: &Option<String>) -> SyncTarget {
    if let Some(t) = target {
        // Accept any backend name directly
        SyncTarget::Backend(Backend::from(t.as_str()))
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
