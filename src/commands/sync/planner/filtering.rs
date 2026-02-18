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
