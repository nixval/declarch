use super::file_ops::rollback_install_edits;
use super::{cleanup_install_backups, plan_installs};
use crate::config::editor::{ModuleEdit, backup_kdl_file};
use std::fs;
use tempfile::tempdir;

#[test]
fn plan_installs_accepts_backend_prefix_per_package() {
    let raw = vec![
        "aur:bat".to_string(),
        "flatpak:org.mozilla.firefox".to_string(),
    ];
    let planned = plan_installs(&raw, None).expect("planning should succeed");
    assert_eq!(planned.len(), 2);
    assert_eq!(planned[0].backend, "aur");
    assert_eq!(planned[0].package, "bat");
    assert_eq!(planned[1].backend, "flatpak");
}

#[test]
fn plan_installs_accepts_global_backend_flag() {
    let raw = vec!["bat".to_string(), "ripgrep".to_string()];
    let backend = Some("aur".to_string());
    let planned = plan_installs(&raw, backend.as_ref()).expect("planning should succeed");
    assert_eq!(planned.len(), 2);
    assert!(planned.iter().all(|p| p.backend == "aur"));
}

#[test]
fn plan_installs_requires_explicit_backend() {
    let raw = vec!["bat".to_string()];
    let err = plan_installs(&raw, None).expect_err("planning should fail without backend");
    assert!(err.to_string().contains("has no backend"));
}

#[test]
fn plan_installs_rejects_unsafe_package_name() {
    let raw = vec!["aur:bat;rm".to_string()];
    let err = plan_installs(&raw, None).expect_err("unsafe package should fail");
    assert!(err.to_string().to_lowercase().contains("unsafe"));
}

#[test]
fn rollback_restores_file_content_from_backup() {
    let dir = tempdir().expect("tempdir");
    let original = dir.path().join("others.kdl");

    fs::write(&original, "pkg {\n  aur { bat }\n}\n").expect("write original");
    let backup = backup_kdl_file(&original).expect("backup");
    fs::write(&original, "pkg {\n  aur { bat ripgrep }\n}\n").expect("mutate");

    let edit = ModuleEdit {
        file_path: original.clone(),
        packages_added: vec!["ripgrep".to_string()],
        created_new_file: false,
        backup_path: Some(backup),
    };

    rollback_install_edits(&[edit], None);
    let restored = fs::read_to_string(&original).expect("read restored");
    assert!(!restored.contains("ripgrep"));
}

#[test]
fn cleanup_removes_backup_file() {
    let dir = tempdir().expect("tempdir");
    let original = dir.path().join("others.kdl");
    fs::write(&original, "pkg { aur { bat } }").expect("write");
    let backup = backup_kdl_file(&original).expect("backup");

    let edit = ModuleEdit {
        file_path: original,
        packages_added: vec![],
        created_new_file: false,
        backup_path: Some(backup.clone()),
    };

    cleanup_install_backups(&[edit], None);
    assert!(!backup.exists());
}
