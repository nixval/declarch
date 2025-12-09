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

// Helper to generate consistent state keys
fn make_state_key(pkg: &PackageId) -> String {
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
        to_update_meta: vec![],
    };

    let target_packages = resolve_target_scope(config, target);
    
    // Suffixes for smart matching
    let suffixes = ["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];

    for pkg_id in target_packages {
        if config.excludes.contains(&pkg_id.name) {
            continue;
        }

        // 1. Try to find exact match in system
        let mut found_meta = installed_snapshot.get(&pkg_id);
        
        // 2. Smart Suffix/Prefix Logic
        if found_meta.is_none() && pkg_id.backend == Backend::Aur {
            // Strategy A: Check Suffixes
            for suffix in suffixes {
                let alt_name = format!("{}{}", pkg_id.name, suffix);
                let alt_id = PackageId {
                    name: alt_name,
                    backend: Backend::Aur,
                };
                if let Some(meta) = installed_snapshot.get(&alt_id) {
                    found_meta = Some(meta);
                    break; 
                }
            }

            // Strategy B: Prefix Fallback
            if found_meta.is_none() {
                 if let Some((prefix, _)) = pkg_id.name.split_once('-') {
                     let alt_id = PackageId {
                         name: prefix.to_string(),
                         backend: Backend::Aur,
                     };
                     if let Some(meta) = installed_snapshot.get(&alt_id) {
                         found_meta = Some(meta);
                     }
                 }
            }
        }

        let state_key = make_state_key(&pkg_id);
        let state_pkg = state.packages.get(&state_key);

        if let Some(meta) = found_meta {
            if let Some(stored_state) = state_pkg {
                if stored_state.version != meta.version {
                    tx.to_update_meta.push(pkg_id.clone());
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
            let core_backend = match state_pkg.backend {
                crate::state::types::Backend::Aur => Backend::Aur,
                crate::state::types::Backend::Flatpak => Backend::Flatpak,
            };

            let name_part = key.split_once(':').map(|(_, n)| n).unwrap_or(&key).to_string();

            let pkg_id = PackageId {
                name: name_part.clone(),
                backend: core_backend,
            };

            // Standard Logic: If not in config, PRUNE IT.
            // We removed the Safety Net here so it flows to Sync, 
            // where we can decide to "Forget" instead of "Destroy".
            if !config.packages.contains_key(&pkg_id) && !config.excludes.contains(&name_part) {
                tx.to_prune.push(pkg_id);
            }
        }
    }

    Ok(tx)
}

// ... resolve_target_scope remains unchanged ...
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
