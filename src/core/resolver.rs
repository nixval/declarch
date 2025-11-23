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

    let target_packages = resolve_target_scope(config, target);

    for pkg_id in target_packages {
        if config.excludes.contains(&pkg_id.name) {
            continue;
        }

        let state_pkg = state.packages.get(&pkg_id.name);
        let system_meta = installed_snapshot.get(&pkg_id);

        if let Some(meta) = system_meta {
            if let Some(stored_state) = state_pkg {
                let stored_backend = match stored_state.backend {
                    crate::state::types::Backend::Aur => Backend::Aur,
                    crate::state::types::Backend::Flatpak => Backend::Flatpak,
                };

                if stored_backend == pkg_id.backend {
                    if stored_state.version != meta.version {
                        tx.to_update_meta.push(pkg_id.clone());
                    }
                } else {
                    tx.to_adopt.push(pkg_id.clone());
                }
            } else {
                tx.to_adopt.push(pkg_id.clone());
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
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{PackageState, Backend as StateBackend};
    use std::path::PathBuf;
    use chrono::Utc; 

    fn mock_config(pkgs: Vec<(&str, Backend)>) -> MergedConfig {
        let mut map = HashMap::new();
        for (name, backend) in pkgs {
            let id = PackageId { name: name.to_string(), backend };
            map.insert(id, vec![PathBuf::from("dummy.kdl")]);
        }
        MergedConfig {
            packages: map,
            excludes: vec![],
        }
    }

    fn mock_state(pkgs: Vec<(&str, StateBackend, &str)>) -> State {
        let mut state = State::default();
        for (name, backend, version) in pkgs {
            state.packages.insert(name.to_string(), PackageState {
                backend,
                installed_at: Utc::now(),
                version: Some(version.to_string()),
            });
        }
        state
    }

    fn mock_snapshot(pkgs: Vec<(&str, Backend, &str)>) -> HashMap<PackageId, PackageMetadata> {
        let mut map = HashMap::new();
        for (name, backend, version) in pkgs {
            let id = PackageId { name: name.to_string(), backend };
            map.insert(id, PackageMetadata {
                version: Some(version.to_string()),
                installed_at: Utc::now(),
                source_file: None,
            });
        }
        map
    }

    #[test]
    fn test_install_flow() {
        let config = mock_config(vec![("git", Backend::Aur)]);
        let state = State::default();
        let snapshot = HashMap::new();
        
        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();
        
        assert_eq!(tx.to_install.len(), 1);
        assert_eq!(tx.to_install[0].name, "git");
        assert!(tx.to_adopt.is_empty());
    }

    #[test]
    fn test_adopt_flow() {
        let config = mock_config(vec![("vim", Backend::Aur)]);
        let state = State::default();
        let snapshot = mock_snapshot(vec![("vim", Backend::Aur, "1.0")]);

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert!(tx.to_install.is_empty());
        assert_eq!(tx.to_adopt.len(), 1);
        assert_eq!(tx.to_adopt[0].name, "vim");
    }

    #[test]
    fn test_prune_safety() {
        let config = MergedConfig::default();
        let state = mock_state(vec![("htop", StateBackend::Aur, "1.0")]);
        let snapshot = mock_snapshot(vec![("htop", Backend::Aur, "1.0")]);

        let tx_all = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();
        assert_eq!(tx_all.to_prune.len(), 1);

        let tx_partial = resolve(&config, &state, &snapshot, &SyncTarget::Named("foo".into())).unwrap();
        assert!(tx_partial.to_prune.is_empty());
    }

    #[test]
    fn test_drift_detection() {
        let config = mock_config(vec![("curl", Backend::Aur)]);
        let state = mock_state(vec![("curl", StateBackend::Aur, "1.0")]); 
        let snapshot = mock_snapshot(vec![("curl", Backend::Aur, "1.2")]); 

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert_eq!(tx.to_update_meta.len(), 1);
        assert_eq!(tx.to_update_meta[0].name, "curl");
        assert!(tx.to_install.is_empty());
        assert!(tx.to_adopt.is_empty());
    }

    #[test]
    fn test_target_backend() {
        let config = mock_config(vec![
            ("git", Backend::Aur),
            ("obsidian", Backend::Flatpak)
        ]);
        let state = State::default();
        let snapshot = HashMap::new();

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::Backend(Backend::Flatpak)).unwrap();

        assert_eq!(tx.to_install.len(), 1);
        assert_eq!(tx.to_install[0].name, "obsidian");
    }
}
