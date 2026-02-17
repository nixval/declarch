use crate::core::types::PackageId;
use crate::state;
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn find_orphans<'a>(
    installed: Vec<&'a state::types::PackageState>,
    config: &HashMap<PackageId, Vec<PathBuf>>,
) -> Vec<&'a state::types::PackageState> {
    installed
        .into_iter()
        .filter(|pkg| {
            let pkg_id = PackageId {
                backend: pkg.backend.clone(),
                name: pkg.config_name.clone(),
            };
            !config.contains_key(&pkg_id)
        })
        .collect()
}

pub(super) fn find_synced<'a>(
    installed: Vec<&'a state::types::PackageState>,
    config: &HashMap<PackageId, Vec<PathBuf>>,
) -> Vec<&'a state::types::PackageState> {
    installed
        .into_iter()
        .filter(|pkg| {
            let pkg_id = PackageId {
                backend: pkg.backend.clone(),
                name: pkg.config_name.clone(),
            };
            config.contains_key(&pkg_id)
        })
        .collect()
}
