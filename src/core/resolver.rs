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
mod tests {
    use super::*;
    use crate::core::types::Backend;
    use crate::state::types::PackageState;
    use chrono::Utc;
    use std::path::PathBuf;

    // Helper: Mock Config
    fn mock_config(pkgs: Vec<(&str, &str)>) -> MergedConfig {
        let mut map = HashMap::new();
        for (name, backend_str) in pkgs {
            let id = PackageId {
                name: name.to_string(),
                backend: Backend::from(backend_str),
            };
            map.insert(id, vec![PathBuf::from("dummy.kdl")]);
        }
        MergedConfig {
            packages: map,
            excludes: vec![],
            project_metadata: None,
            conflicts: vec![],
            backend_options: std::collections::HashMap::new(),
            env: std::collections::HashMap::new(),
            package_sources: std::collections::HashMap::new(),
            policy: None,
            lifecycle_actions: None,
            editor: None,
            backends: vec![],
            backend_sources: std::collections::HashMap::new(),
            experimental: std::collections::HashSet::new(),
        }
    }

    // Helper: Mock State (Updated to use new "backend:name" key format)
    fn mock_state(pkgs: Vec<(&str, &str, &str)>) -> State {
        let mut state = State::default();
        for (name, backend_str, version) in pkgs {
            let backend = Backend::from(backend_str);
            let id = PackageId {
                name: name.to_string(),
                backend: backend.clone(),
            };
            let key = make_state_key(&id);

            state.packages.insert(
                key,
                PackageState {
                    backend: backend.clone(),
                    config_name: name.to_string(),
                    provides_name: name.to_string(),
                    actual_package_name: None,
                    installed_at: Utc::now(),
                    version: Some(version.to_string()),
                },
            );
        }
        state
    }

    // Helper: Mock System Snapshot
    fn mock_snapshot(pkgs: Vec<(&str, &str, &str)>) -> HashMap<PackageId, PackageMetadata> {
        let mut map = HashMap::new();
        for (name, backend_str, version) in pkgs {
            let id = PackageId {
                name: name.to_string(),
                backend: Backend::from(backend_str),
            };
            map.insert(
                id,
                PackageMetadata {
                    version: Some(version.to_string()),
                    variant: None,
                    installed_at: Utc::now(),
                    source_file: None,
                },
            );
        }
        map
    }

    #[test]
    fn test_install_flow() {
        // Case: Config has git, System empty -> Install git
        let config = mock_config(vec![("git", "aur")]);
        let state = State::default();
        let snapshot = HashMap::new();

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert_eq!(tx.to_install.len(), 1);
        assert_eq!(tx.to_install[0].name, "git");
    }

    #[test]
    fn test_smart_match_suffix() {
        // Case: Config has "gdu", System has "gdu-bin" -> Adopt "gdu" (Mapped)
        let config = mock_config(vec![("gdu", "aur")]);
        let state = State::default();
        let snapshot = mock_snapshot(vec![("gdu-bin", "aur", "1.0")]);

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert!(tx.to_install.is_empty());
        assert_eq!(tx.to_adopt.len(), 1);
        assert_eq!(tx.to_adopt[0].name, "gdu"); // We adopt the Config Name
    }

    #[test]
    fn test_smart_match_prefix() {
        // Case: Config has "rofi-wayland", System has "rofi" -> Adopt
        let config = mock_config(vec![("rofi-wayland", "aur")]);
        let state = State::default();
        let snapshot = mock_snapshot(vec![("rofi", "aur", "2.0")]);

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert!(tx.to_install.is_empty());
        assert_eq!(tx.to_adopt.len(), 1);
        assert_eq!(tx.to_adopt[0].name, "rofi-wayland");
    }

    #[test]
    fn test_prune_logic_standard() {
        // Case: Config empty, State has "htop" -> Prune htop
        let config = MergedConfig::default();
        let state = mock_state(vec![("htop", "aur", "1.0")]);
        let snapshot = mock_snapshot(vec![("htop", "aur", "1.0")]);

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();
        assert_eq!(tx.to_prune.len(), 1);
        assert_eq!(tx.to_prune[0].name, "htop");
    }

    #[test]
    fn test_flatpak_fuzzy() {
        // Case: Config "spotify", System "com.spotify.Client" -> Adopt
        let config = mock_config(vec![("spotify", "flatpak")]);
        let state = State::default();
        let snapshot = mock_snapshot(vec![("com.spotify.Client", "flatpak", "1.0")]);

        let tx = resolve(&config, &state, &snapshot, &SyncTarget::All).unwrap();

        assert!(tx.to_install.is_empty());
        assert_eq!(tx.to_adopt.len(), 1);
        assert_eq!(tx.to_adopt[0].name, "spotify");
    }
}
