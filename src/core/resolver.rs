use crate::config::loader::MergedConfig;
use crate::state::types::State;
use crate::core::types::{Backend, PackageId, PackageMetadata, SyncTarget};
use crate::error::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Transaction {
    pub to_install: Vec<PackageId>,
    pub to_prune: Vec<PackageId>,
    pub to_adopt: Vec<PackageId>,
    pub to_update_meta: Vec<PackageId>,
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
        to_update_meta: vec![],
    };

    // 1. Filter Target (Scope Resolution)
    let target_packages = resolve_target_scope(config, target);

    // 2. Calculate Diff
    for pkg_id in target_packages {
        // Cek Excludes
        if config.excludes.contains(&pkg_id.name) {
            continue;
        }

        let is_managed = state.packages.contains_key(&pkg_id.name); 
        let is_installed = installed_snapshot.contains_key(&pkg_id);

        if is_installed {
            if !is_managed {
                tx.to_adopt.push(pkg_id.clone());
            } else {
                if let Some(_meta) = installed_snapshot.get(&pkg_id) {
                    tx.to_update_meta.push(pkg_id.clone());
                }
            }
        } else {
            tx.to_install.push(pkg_id.clone());
        }
    }

    if *target == SyncTarget::All {
for (name, state_pkg) in &state.packages {
            
         
            let core_backend = match state_pkg.backend {
                crate::state::types::Backend::Aur => Backend::Aur,
                crate::state::types::Backend::Flatpak => Backend::Flatpak,
            };

            let pkg_id = PackageId {
                name: name.clone(),
                backend: core_backend, 
            };

            if !config.packages.contains_key(&pkg_id) && !config.excludes.contains(name) {
                tx.to_prune.push(pkg_id);
            }
        }    }

    Ok(tx)
}

fn resolve_target_scope(config: &MergedConfig, target: &SyncTarget) -> HashSet<PackageId> {
    match target {
        SyncTarget::All => config.packages.keys().cloned().collect(),
        
        SyncTarget::Backend(backend) => config.packages.keys()
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
                    if let Some(stem) = source.file_stem() {
                        if stem.to_string_lossy().to_lowercase() == query_lower {
                            matched.insert(pkg_id.clone());
                        }
                    }
                }
            }
            matched
        }
    }
}
