use crate::config::loader;
use crate::core::{resolver, types::SyncTarget};
use crate::error::Result;
use crate::project_identity;
use crate::state::types::State;
use crate::ui as output;

use super::{InstalledSnapshot, ManagerMap};

pub(super) fn resolve_filtered_transaction(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    use crate::core::types::Backend;
    use std::collections::{HashMap, HashSet};

    let available_backends: HashSet<Backend> = managers.keys().cloned().collect();
    let total_packages = config.packages.len();

    let filtered_packages: HashMap<_, _> = config
        .packages
        .iter()
        .filter(|(pkg_id, _)| available_backends.contains(&pkg_id.backend))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let skipped_count = total_packages - filtered_packages.len();
    if skipped_count > 0 {
        let mut skipped_by_backend: HashMap<String, usize> = HashMap::new();
        for pkg_id in config.packages.keys() {
            if !available_backends.contains(&pkg_id.backend) {
                *skipped_by_backend
                    .entry(pkg_id.backend.to_string())
                    .or_insert(0) += 1;
            }
        }

        for (backend, count) in skipped_by_backend {
            output::warning(&format!(
                "Skipping {} package(s), backend '{}' not available. Run '{}'",
                count,
                backend,
                project_identity::cli_with(&format!("init --backend {}", backend))
            ));
        }
    }

    let original_packages = std::mem::replace(&mut config.packages, filtered_packages);
    let tx = resolver::resolve(config, state, installed_snapshot, sync_target)?;
    config.packages = original_packages;

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::resolve_filtered_transaction;
    use crate::commands::sync::ManagerMap;
    use crate::config::loader;
    use crate::core::types::{Backend, PackageId, SyncTarget};
    use crate::error::Result;
    use crate::packages::traits::PackageManager;
    use crate::state::types::State;
    use std::collections::HashMap;

    struct DummyManager {
        backend: Backend,
    }

    impl PackageManager for DummyManager {
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
            Ok(())
        }
        fn is_available(&self) -> bool {
            true
        }
        fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn resolve_filtered_transaction_only_targets_available_backends() {
        let mut config = loader::MergedConfig::default();
        config.packages.insert(
            PackageId {
                name: "bat".to_string(),
                backend: Backend::from("aur"),
            },
            vec![],
        );
        config.packages.insert(
            PackageId {
                name: "hello".to_string(),
                backend: Backend::from("brew"),
            },
            vec![],
        );

        let mut managers: ManagerMap = HashMap::new();
        managers.insert(
            Backend::from("aur"),
            Box::new(DummyManager {
                backend: Backend::from("aur"),
            }),
        );

        let original_len = config.packages.len();
        let tx = resolve_filtered_transaction(
            &mut config,
            &State::default(),
            &HashMap::new(),
            &managers,
            &SyncTarget::All,
        )
        .expect("resolve should succeed");

        assert_eq!(tx.to_install.len(), 1);
        assert!(tx.to_install.iter().any(|p| p.name == "bat"));
        assert_eq!(config.packages.len(), original_len);
    }
}
