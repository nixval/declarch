use super::{load_state_from_path, sanitize_state_in_place, validate_state_integrity};
use crate::project_identity;
use crate::state::types::{Backend, PackageState, State};
use chrono::Utc;
use std::fs;
use tempfile::tempdir;

#[test]
fn sanitize_removes_empty_and_rekeys() {
    let mut state = State::default();
    state.packages.insert(
        "wrong:key".to_string(),
        PackageState {
            backend: Backend::from("aur"),
            config_name: "bat".to_string(),
            provides_name: String::new(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: Some("1.0".to_string()),
            install_reason: None,
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );
    state.packages.insert(
        "aur:empty".to_string(),
        PackageState {
            backend: Backend::from("aur"),
            config_name: String::new(),
            provides_name: String::new(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: None,
            install_reason: None,
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );

    let report = sanitize_state_in_place(&mut state);
    assert_eq!(report.removed_empty_name, 1);
    assert_eq!(report.rekeyed_entries, 1);
    assert_eq!(report.normalized_fields, 1);
    assert!(state.packages.contains_key("aur:bat"));
}

#[test]
fn validate_flags_non_canonical_keys() {
    let mut state = State::default();
    state.packages.insert(
        "bad:key".to_string(),
        PackageState {
            backend: Backend::from("aur"),
            config_name: "bat".to_string(),
            provides_name: "bat".to_string(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: None,
            install_reason: None,
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );
    let issues = validate_state_integrity(&state);
    assert!(issues.iter().any(|i| i.contains("Non-canonical state key")));
}

#[test]
fn load_state_non_strict_falls_back_to_default_when_unrecoverable() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("state.json");
    fs::write(&path, "{broken json").expect("write corrupted state");

    let loaded = load_state_from_path(&path, false).expect("non-strict should not fail");
    assert!(loaded.packages.is_empty());
}

#[test]
fn load_state_strict_fails_when_unrecoverable() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("state.json");
    fs::write(&path, "{broken json").expect("write corrupted state");

    let err = load_state_from_path(&path, true).expect_err("strict mode should fail");
    let msg = err.to_string();
    assert!(msg.contains("strict mode"));
    assert!(msg.contains(&format!("File: {}", path.display())));
    assert!(msg.contains(&format!(
        "Hint: run `{}`",
        project_identity::cli_with("info --doctor")
    )));
}

#[test]
fn load_state_strict_restores_from_backup_when_available() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("state.json");
    fs::write(&path, "{broken json").expect("write corrupted state");

    let backup_path = dir.path().join("state.json.bak.1");
    let backup_content = serde_json::to_string(&State::default()).expect("serialize default state");
    fs::write(&backup_path, backup_content).expect("write backup");

    let loaded = load_state_from_path(&path, true).expect("strict mode should recover");
    assert!(loaded.packages.is_empty());
}
