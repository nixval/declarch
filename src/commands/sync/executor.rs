//! Transaction execution for sync operations
//!
//! Installs, adopts, and prunes packages based on transaction plan.

mod prune;
mod retry;
mod snapshot;

use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::commands::sync::hooks::{execute_post_install, execute_pre_install};
use crate::config::loader;
use crate::constants::{BACKEND_OPERATION_MAX_RETRIES, BACKEND_RETRY_DELAY_MS};
use crate::core::{
    resolver,
    types::{Backend, PackageId},
};
use crate::error::Result;
use crate::ui as output;
use prune::execute_pruning;
use retry::execute_with_retry;
use snapshot::build_installed_snapshot;
use std::collections::{HashMap, HashSet};

/// Maximum retry attempts for failed backend operations
const MAX_RETRIES: u32 = BACKEND_OPERATION_MAX_RETRIES;
/// Delay between retries (in milliseconds)
const RETRY_DELAY_MS: u64 = BACKEND_RETRY_DELAY_MS;

/// Execute transaction (install, adopt, prune)
pub fn execute_transaction(
    transaction: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
    hooks_enabled: bool,
) -> Result<Vec<PackageId>> {
    let mut installed_snapshot = build_installed_snapshot(managers)?;

    // Execute installations
    let successfully_installed = execute_installations(
        transaction,
        managers,
        config,
        options,
        hooks_enabled,
        &mut installed_snapshot,
    )?;

    // Execute pruning if enabled
    if options.prune && !transaction.to_prune.is_empty() {
        execute_pruning(
            config,
            transaction,
            managers,
            options,
            hooks_enabled,
            &installed_snapshot,
        )?;
    }

    Ok(successfully_installed)
}

/// Execute package installations
fn execute_installations(
    tx: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
    hooks_enabled: bool,
    installed_snapshot: &mut InstalledSnapshot,
) -> Result<Vec<PackageId>> {
    // Group packages by backend
    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install.iter() {
        installs
            .entry(pkg.backend.clone())
            .or_default()
            .push(pkg.name.clone());
    }

    let mut successfully_installed = Vec::new();

    // Install packages with retry logic
    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend));

            for pkg_name in &pkgs {
                execute_pre_install(
                    &config.lifecycle_actions,
                    pkg_name,
                    hooks_enabled,
                    options.dry_run,
                )?;
            }

            // Track which packages exist before installation
            let pre_install_snapshot: HashSet<_> = match mgr.list_installed() {
                Ok(pkgs) => pkgs.keys().cloned().collect(),
                Err(e) => {
                    output::error(&format!(
                        "Failed to list installed packages for {}: {}",
                        backend, e
                    ));
                    continue;
                }
            };

            // Try installation with retries
            let install_result = execute_with_retry(
                || mgr.install(&pkgs),
                &format!("install packages for {}", backend),
                MAX_RETRIES,
                RETRY_DELAY_MS,
            );

            if let Err(e) = install_result {
                output::error(&format!(
                    "Failed to install packages for {}: {}",
                    backend, e
                ));
                output::info("Continuing with other backends...");
                continue;
            }

            // Check which packages exist after installation
            let post_install_snapshot: HashSet<_> = match mgr.list_installed() {
                Ok(pkgs) => pkgs.keys().cloned().collect(),
                Err(e) => {
                    output::warning(&format!(
                        "Failed to verify installation for {}: {}",
                        backend, e
                    ));
                    // Assume all packages were installed
                    for pkg_name in &pkgs {
                        execute_post_install(
                            &config.lifecycle_actions,
                            pkg_name,
                            hooks_enabled,
                            options.dry_run,
                        )?;
                        successfully_installed.push(PackageId {
                            name: pkg_name.clone(),
                            backend: backend.clone(),
                        });
                    }
                    continue;
                }
            };

            // Find newly installed packages
            for pkg_name in &pkgs {
                if !pre_install_snapshot.contains(pkg_name)
                    && post_install_snapshot.contains(pkg_name)
                {
                    execute_post_install(
                        &config.lifecycle_actions,
                        pkg_name,
                        hooks_enabled,
                        options.dry_run,
                    )?;
                    successfully_installed.push(PackageId {
                        name: pkg_name.clone(),
                        backend: backend.clone(),
                    });
                }
            }
        }
    }

    // Refresh snapshot after installations
    if !tx.to_install.is_empty() && !successfully_installed.is_empty() {
        // Only show message if packages were actually installed
        output::info(&format!(
            "Installed {} package(s)",
            successfully_installed.len()
        ));

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

    Ok(successfully_installed)
}
