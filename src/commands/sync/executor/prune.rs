use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::commands::sync::hooks::{execute_post_remove, execute_pre_remove};
use crate::commands::sync::variants::resolve_installed_package_name;
use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{resolver, types::Backend};
use crate::error::Result;
use crate::ui as output;
use colored::Colorize;
use std::collections::{HashMap, HashSet};

pub(super) fn execute_pruning(
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

    let mut protected_physical_names: Vec<String> = Vec::new();
    for pkg in config.packages.keys() {
        if config.excludes.contains(&pkg.name) {
            continue;
        }
        let real_name = resolve_installed_package_name(pkg, installed_snapshot);
        protected_physical_names.push(real_name);
    }

    let mut removes: HashMap<Backend, Vec<String>> = HashMap::new();
    let mut remove_hooks: HashMap<Backend, Vec<(String, String)>> = HashMap::new();
    let policy_protected: HashSet<String> = config
        .policy
        .as_ref()
        .map(|p| p.protected.iter().cloned().collect())
        .unwrap_or_default();

    for pkg in tx.to_prune.iter() {
        if CRITICAL_PACKAGES.contains(&pkg.name.as_str()) || policy_protected.contains(&pkg.name) {
            continue;
        }

        execute_pre_remove(
            &config.lifecycle_actions,
            &pkg.name,
            hooks_enabled,
            options.dry_run,
        )?;

        let real_name = resolve_installed_package_name(pkg, installed_snapshot);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PackageId;
    use crate::packages::traits::PackageManager;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockManager {
        backend: Backend,
        remove_calls: AtomicUsize,
        remove_error: Option<String>,
    }

    impl PackageManager for MockManager {
        fn backend_type(&self) -> Backend {
            self.backend.clone()
        }

        fn list_installed(&self) -> Result<HashMap<String, crate::core::types::PackageMetadata>> {
            Ok(HashMap::new())
        }

        fn install(&self, _packages: &[String]) -> Result<()> {
            Ok(())
        }

        fn remove(&self, _packages: &[String]) -> Result<()> {
            self.remove_calls.fetch_add(1, Ordering::SeqCst);
            if let Some(msg) = &self.remove_error {
                return Err(crate::error::DeclarchError::PackageManagerError(
                    msg.clone(),
                ));
            }
            Ok(())
        }

        fn is_available(&self) -> bool {
            true
        }

        fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
    }

    fn base_options() -> SyncOptions {
        SyncOptions {
            dry_run: false,
            prune: true,
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

    #[test]
    fn prune_skips_when_policy_orphans_keep() {
        let mut config = loader::MergedConfig::default();
        config.policy = Some(crate::config::kdl::PolicyConfig {
            orphans: Some("keep".to_string()),
            ..Default::default()
        });
        let tx = resolver::Transaction {
            to_install: Vec::new(),
            to_prune: vec![PackageId {
                name: "bat".to_string(),
                backend: Backend::from("aur"),
            }],
            to_adopt: Vec::new(),
            to_update_project_metadata: Vec::new(),
        };
        let managers: ManagerMap = HashMap::new();
        let snapshot = InstalledSnapshot::new();

        let out = execute_pruning(&config, &tx, &managers, &base_options(), false, &snapshot);
        assert!(out.is_ok());
    }

    #[test]
    fn prune_ignores_backends_without_remove_support() {
        let mut config = loader::MergedConfig::default();
        config.policy = Some(crate::config::kdl::PolicyConfig {
            orphans: Some("remove".to_string()),
            ..Default::default()
        });
        let backend = Backend::from("aur");
        let tx = resolver::Transaction {
            to_install: Vec::new(),
            to_prune: vec![PackageId {
                name: "bat".to_string(),
                backend: backend.clone(),
            }],
            to_adopt: Vec::new(),
            to_update_project_metadata: Vec::new(),
        };

        let manager = MockManager {
            backend: backend.clone(),
            remove_calls: AtomicUsize::new(0),
            remove_error: Some("backend does not support removing".to_string()),
        };
        let mut managers: ManagerMap = HashMap::new();
        managers.insert(backend, Box::new(manager));
        let snapshot = InstalledSnapshot::new();

        let out = execute_pruning(&config, &tx, &managers, &base_options(), false, &snapshot);
        assert!(out.is_ok());
    }
}
