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
        mcp: None,
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
                install_reason: None,
                source_module: None,
                last_seen_at: None,
                backend_meta: None,
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
