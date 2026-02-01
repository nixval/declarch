//! Transaction execution for sync operations
//!
//! Installs, adopts, and prunes packages based on transaction plan.

use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{resolver, types::{Backend, PackageId, PackageMetadata}};
use crate::error::Result;
use crate::ui as output;
use super::{ManagerMap, SyncOptions, InstalledSnapshot, resolve_installed_package_name};
use colored::Colorize;
use std::collections::{HashMap, HashSet};

/// Execute transaction (install, adopt, prune)
pub fn execute_transaction(
    transaction: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
) -> Result<Vec<PackageId>> {
    let mut installed_snapshot: InstalledSnapshot = HashMap::new();

    // Build initial snapshot from managers
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

    // Execute installations
    let successfully_installed = execute_installations(
        transaction,
        managers,
        &mut installed_snapshot,
    )?;

    // Execute pruning if enabled
    if options.prune && !transaction.to_prune.is_empty() {
        execute_pruning(
            config,
            transaction,
            managers,
            &installed_snapshot,
        )?;
    }

    Ok(successfully_installed)
}

/// Execute package installations
fn execute_installations(
    tx: &resolver::Transaction,
    managers: &ManagerMap,
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

    // Install packages
    for (backend, pkgs) in installs {
        if let Some(mgr) = managers.get(&backend) {
            output::info(&format!("Installing {} packages...", backend));

            // Track which packages exist before installation
            let pre_install_snapshot: HashSet<_> = mgr.list_installed()?.keys().cloned().collect();

            mgr.install(&pkgs)?;

            // Check which packages exist after installation
            let post_install_snapshot: HashSet<_> = mgr.list_installed()?.keys().cloned().collect();

            // Find newly installed packages
            for pkg_name in &pkgs {
                if !pre_install_snapshot.contains(pkg_name)
                    && post_install_snapshot.contains(pkg_name)
                {
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
