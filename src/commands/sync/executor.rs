//! Transaction execution for sync operations
//!
//! Installs, adopts, and prunes packages based on transaction plan.

use super::variants::resolve_installed_package_name;
use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::commands::sync::hooks::{
    execute_post_install, execute_post_remove, execute_pre_install, execute_pre_remove,
};
use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{
    resolver,
    types::{Backend, PackageId},
};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use colored::Colorize;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

/// Maximum retry attempts for failed backend operations
const MAX_RETRIES: u32 = 3;
/// Delay between retries (in milliseconds)
const RETRY_DELAY_MS: u64 = 1000;

/// Execute a function with retry logic
fn execute_with_retry<F>(
    mut operation: F,
    operation_name: &str,
    max_retries: u32,
    delay_ms: u64,
) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match operation() {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    output::warning(&format!(
                        "{} failed (attempt {}/{}), retrying in {}s...",
                        operation_name,
                        attempt,
                        max_retries,
                        delay_ms / 1000
                    ));
                    thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        DeclarchError::Other(format!(
            "{} failed after {} attempts",
            operation_name, max_retries
        ))
    }))
}

/// Execute transaction (install, adopt, prune)
pub fn execute_transaction(
    transaction: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
    hooks_enabled: bool,
) -> Result<Vec<PackageId>> {
    // Build initial snapshot from managers in parallel
    // This significantly speeds up sync when multiple backends are configured
    let installed_snapshot: InstalledSnapshot = managers
        .par_iter()
        .filter_map(|(backend, mgr)| {
            if !mgr.is_available() {
                return None;
            }
            match mgr.list_installed() {
                Ok(packages) => {
                    let packages_with_backend: Vec<_> = packages
                        .into_iter()
                        .map(|(name, meta)| {
                            let id = PackageId {
                                name,
                                backend: backend.clone(),
                            };
                            (id, meta)
                        })
                        .collect();
                    Some(Ok(packages_with_backend))
                }
                Err(e) => {
                    output::warning(&format!("Failed to list packages for {}: {}", backend, e));
                    Some(Err(e))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    let mut installed_snapshot = installed_snapshot;

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

/// Execute package pruning with safety checks
fn execute_pruning(
    config: &loader::MergedConfig,
    tx: &resolver::Transaction,
    managers: &ManagerMap,
    options: &SyncOptions,
    hooks_enabled: bool,
    installed_snapshot: &InstalledSnapshot,
) -> Result<()> {
    let orphan_strategy = config
        .policy
        .as_ref()
        .and_then(|p| p.orphans.clone())
        .unwrap_or_else(|| "remove".to_string())
        .to_lowercase();

    if orphan_strategy == "keep" {
        output::info("Skipping orphan removal (policy.orphans = \"keep\")");
        return Ok(());
    }

    if orphan_strategy == "ask"
        && !options.yes
        && !output::prompt_yes_no("Policy requests confirmation for orphan removal. Continue?")
    {
        output::info("Skipping orphan removal");
        return Ok(());
    }

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
    let mut remove_hooks: HashMap<Backend, Vec<(String, String)>> = HashMap::new();
    let policy_protected: HashSet<String> = config
        .policy
        .as_ref()
        .map(|p| p.protected.iter().cloned().collect())
        .unwrap_or_default();

    for pkg in tx.to_prune.iter() {
        // 1. GHOST MODE (Static Check) - Skip critical packages
        if CRITICAL_PACKAGES.contains(&pkg.name.as_str()) || policy_protected.contains(&pkg.name) {
            continue;
        }

        execute_pre_remove(
            &config.lifecycle_actions,
            &pkg.name,
            hooks_enabled,
            options.dry_run,
        )?;

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
            .push(real_name.clone());
        remove_hooks
            .entry(pkg.backend.clone())
            .or_default()
            .push((real_name, pkg.name.clone()));
    }

    // Execute removals
    for (backend, pkgs) in removes {
        if !pkgs.is_empty()
            && let Some(mgr) = managers.get(&backend)
        {
            output::info(&format!("Removing {} packages...", backend));
            match mgr.remove(&pkgs) {
                Ok(()) => {
                    if let Some(hook_entries) = remove_hooks.get(&backend) {
                        for (_, config_name) in hook_entries {
                            execute_post_remove(
                                &config.lifecycle_actions,
                                config_name,
                                hooks_enabled,
                                options.dry_run,
                            )?;
                        }
                    } else {
                        for pkg_name in &pkgs {
                            execute_post_remove(
                                &config.lifecycle_actions,
                                pkg_name,
                                hooks_enabled,
                                options.dry_run,
                            )?;
                        }
                    }
                }
                Err(e) => {
                    // Check if this is a "not supported" error
                    let error_msg = format!("{}", e);
                    if error_msg.contains("does not support removing") {
                        output::warning(&format!(
                            "Cannot remove {} package(s) - backend '{}' does not support removal",
                            pkgs.len(),
                            backend
                        ));
                        output::info(&format!("Packages not removed: {}", pkgs.join(", ")));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}
