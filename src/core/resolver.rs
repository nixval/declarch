use crate::config::loader::MergedConfig;
use crate::core::matcher::PackageMatcher;
use crate::core::types::{PackageId, PackageMetadata, SyncTarget};
use crate::error::Result;
use crate::state::types::State;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Transaction {
    pub to_install: Vec<PackageId>,
    pub to_prune: Vec<PackageId>,
    pub to_adopt: Vec<PackageId>,
    pub to_update_project_metadata: Vec<PackageId>,
}

/// Helper to generate consistent state keys
/// Public function to use across modules for consistency
pub fn make_state_key(pkg: &PackageId) -> String {
    format!("{}:{}", pkg.backend, pkg.name)
}

pub fn resolve(
    config: &MergedConfig,
    state: &State,
    installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    target: &SyncTarget,
) -> Result<Transaction> {
    let mut tx = Transaction {
        to_install: vec![],
        to_prune: vec![],
        to_adopt: vec![],
        to_update_project_metadata: vec![],
    };

    let target_packages = resolve_target_scope(config, target);

    // Create smart matcher for package resolution
    let matcher = PackageMatcher::new();

    for pkg_id in target_packages {
        if config.excludes.contains(&pkg_id.name) {
            continue;
        }

        // Use PackageMatcher to find installed package (handles variants)
        let matched_id = matcher.find_package(&pkg_id, installed_snapshot);
        let found_meta = matched_id
            .as_ref()
            .and_then(|id| installed_snapshot.get(id));

        let state_key = make_state_key(&pkg_id);
        let state_pkg = state.packages.get(&state_key);

        if let Some(meta) = found_meta {
            if let Some(stored_state) = state_pkg {
                if stored_state.version != meta.version {
                    tx.to_update_project_metadata.push(pkg_id.clone());
                }
            } else {
                tx.to_adopt.push(pkg_id.clone());
            }
        } else {
            tx.to_install.push(pkg_id.clone());
        }
    }

    // Pruning Logic
    if *target == SyncTarget::All {
        for (key, state_pkg) in &state.packages {
            // Backend is now the same type from core::types
            let core_backend = state_pkg.backend.clone();

            let name_part = key
                .split_once(':')
                .map(|(_, n)| n)
                .unwrap_or(key)
                .to_string();
            let pkg_id = PackageId {
                name: name_part.clone(),
                backend: core_backend,
            };

            if !config.packages.contains_key(&pkg_id) && !config.excludes.contains(&name_part) {
                tx.to_prune.push(pkg_id);
            }
        }
    }

    Ok(tx)
}

fn resolve_target_scope(config: &MergedConfig, target: &SyncTarget) -> HashSet<PackageId> {
    match target {
        SyncTarget::All => config.packages.keys().cloned().collect(),

        SyncTarget::Backend(backend) => config
            .packages
            .keys()
            .filter(|p| p.backend == *backend)
            .cloned()
            .collect(),

        SyncTarget::Named(query) => {
            let mut matched = HashSet::new();
            let query_lower = query.to_lowercase();

            for (pkg_id, sources) in &config.packages {
                if pkg_id.name == *query {
                    matched.insert(pkg_id.clone());
                }

                for source in sources {
                    if let Some(stem) = source.file_stem()
                        && stem.to_string_lossy().to_lowercase() == query_lower
                    {
                        matched.insert(pkg_id.clone());
                    }
                }
            }
            matched
        }
    }
}

#[cfg(test)]
mod tests;
