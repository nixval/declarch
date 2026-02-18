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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PackageMetadata;
    use crate::packages::traits::PackageManager;
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockManager {
        backend: Backend,
        available: bool,
        install_ok: bool,
        installed_sequence: Mutex<Vec<HashMap<String, PackageMetadata>>>,
    }

    impl PackageManager for MockManager {
        fn backend_type(&self) -> Backend {
            self.backend.clone()
        }

        fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
            let mut guard = self
                .installed_sequence
                .lock()
                .expect("lock installed sequence");
            if guard.is_empty() {
                Ok(HashMap::new())
            } else {
                Ok(guard.remove(0))
            }
        }

        fn install(&self, _packages: &[String]) -> Result<()> {
            if self.install_ok {
                Ok(())
            } else {
                Err(crate::error::DeclarchError::Other(
                    "install failed".to_string(),
                ))
            }
        }

        fn remove(&self, _packages: &[String]) -> Result<()> {
            Ok(())
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
    }

    fn base_options() -> SyncOptions {
        SyncOptions {
            dry_run: false,
            prune: false,
            update: false,
            verbose: false,
            yes: true,
            force: false,
            target: None,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: Vec::new(),
            diff: false,
            format: None,
            output_version: None,
        }
    }

    fn package_meta() -> PackageMetadata {
        PackageMetadata {
            version: Some("1.0".to_string()),
            variant: None,
            installed_at: Utc::now(),
            source_file: None,
        }
    }

    #[test]
    fn execute_installations_marks_newly_installed_packages() {
        let backend = Backend::from("aur");
        let pkg = PackageId {
            name: "bat".to_string(),
            backend: backend.clone(),
        };

        let tx = resolver::Transaction {
            to_install: vec![pkg.clone()],
            to_prune: Vec::new(),
            to_adopt: Vec::new(),
            to_update_project_metadata: Vec::new(),
        };

        let mut pre = HashMap::new();
        pre.insert("ripgrep".to_string(), package_meta());
        let mut post = HashMap::new();
        post.insert("ripgrep".to_string(), package_meta());
        post.insert("bat".to_string(), package_meta());

        let mut managers: ManagerMap = HashMap::new();
        managers.insert(
            backend.clone(),
            Box::new(MockManager {
                backend: backend.clone(),
                available: true,
                install_ok: true,
                installed_sequence: Mutex::new(vec![pre, post]),
            }),
        );

        let config = loader::MergedConfig::default();
        let mut installed_snapshot = InstalledSnapshot::new();
        let installed = execute_installations(
            &tx,
            &managers,
            &config,
            &base_options(),
            false,
            &mut installed_snapshot,
        )
        .expect("installations should succeed");

        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].name, "bat");
    }
}
