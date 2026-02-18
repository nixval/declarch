use super::{InstalledSnapshot, MAX_RETRIES, ManagerMap, RETRY_DELAY_MS, SyncOptions};
use crate::commands::sync::hooks::{execute_post_install, execute_pre_install};
use crate::config::loader;
use crate::core::{
    resolver,
    types::{Backend, PackageId},
};
use crate::error::Result;
use crate::ui as output;
use std::collections::{HashMap, HashSet};

use super::retry::execute_with_retry;

pub(super) fn execute_installations(
    tx: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
    hooks_enabled: bool,
    installed_snapshot: &mut InstalledSnapshot,
) -> Result<Vec<PackageId>> {
    let mut installs: HashMap<Backend, Vec<String>> = HashMap::new();
    for pkg in tx.to_install.iter() {
        installs
            .entry(pkg.backend.clone())
            .or_default()
            .push(pkg.name.clone());
    }

    let mut successfully_installed = Vec::new();

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

            let post_install_snapshot: HashSet<_> = match mgr.list_installed() {
                Ok(pkgs) => pkgs.keys().cloned().collect(),
                Err(e) => {
                    output::warning(&format!(
                        "Failed to verify installation for {}: {}",
                        backend, e
                    ));
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

    if !tx.to_install.is_empty() && !successfully_installed.is_empty() {
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
