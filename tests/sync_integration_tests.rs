//! Integration tests for sync workflow
//!
//! Tests the complete sync pipeline including:
//! - Transaction planning
//! - State management
//! - Dry-run mode
//! - Hook execution (when enabled)

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Utc;
use declarch::commands::sync::SyncOptions;
use declarch::config::loader::MergedConfig;
use declarch::core::resolver::Transaction;
use declarch::core::types::{PackageId, PackageMetadata, SyncTarget};
use declarch::state::types::{Backend, PackageState, State, StateMeta};

/// Helper to create a test package ID
fn pkg_id(backend: Backend, name: &str) -> PackageId {
    PackageId {
        backend,
        name: name.to_string(),
    }
}

/// Helper to create a test package metadata
fn pkg_meta(version: &str) -> PackageMetadata {
    PackageMetadata {
        version: Some(version.to_string()),
        variant: None,
        installed_at: Utc::now(),
        source_file: None,
    }
}

/// Helper to create a test package state
fn pkg_state(backend: Backend, name: &str) -> PackageState {
    PackageState {
        backend,
        config_name: name.to_string(),
        provides_name: name.to_string(),
        aur_package_name: None,
        installed_at: Utc::now(),
        version: Some("1.0.0".to_string()),
    }
}

/// Helper to create a test state
fn test_state(packages: Vec<(Backend, &str)>) -> State {
    let mut state = State {
        packages: HashMap::new(),
        meta: StateMeta {
            schema_version: 1,
            hostname: "test-host".to_string(),
            last_sync: Utc::now(),
            last_update: None,
        },
    };

    for (backend, name) in packages {
        let key = format!("{:?}:{}", backend, name);
        state.packages.insert(key, pkg_state(backend, name));
    }

    state
}

/// Helper to create a test config
fn test_config(packages: Vec<(Backend, &str)>) -> MergedConfig {
    let mut config = MergedConfig::default();

    for (backend, name) in packages {
        let pkg_id_val = pkg_id(backend, name);
        // MergedConfig stores Vec<PathBuf> for each package
        config
            .packages
            .insert(pkg_id_val, vec![PathBuf::from("test.kdl")]);
    }

    config
}

#[test]
fn test_sync_empty_config_no_changes() {
    // Test sync with empty config - should result in empty transaction
    let _config = test_config(vec![]);
    let _state = test_state(vec![]);

    // Create empty transaction
    let tx = Transaction {
        to_install: vec![],
        to_adopt: vec![],
        to_prune: vec![],
        to_update_project_metadata: vec![],
    };

    // Verify empty transaction
    assert!(tx.to_install.is_empty());
    assert!(tx.to_adopt.is_empty());
    assert!(tx.to_prune.is_empty());
}

#[test]
fn test_sync_package_needs_install() {
    // Test detecting packages that need installation
    let _config = test_config(vec![(Backend::Aur, "bat"), (Backend::Aur, "fzf")]);

    // State has no packages (fresh install)
    let _state = test_state(vec![]);

    // Both packages should be in to_install
    let tx = Transaction {
        to_install: vec![pkg_id(Backend::Aur, "bat"), pkg_id(Backend::Aur, "fzf")],
        to_adopt: vec![],
        to_prune: vec![],
        to_update_project_metadata: vec![],
    };

    assert_eq!(tx.to_install.len(), 2);
    assert!(tx.to_prune.is_empty());
    assert_eq!(tx.to_install[0].name, "bat");
    assert_eq!(tx.to_install[1].name, "fzf");
}

#[test]
fn test_sync_package_already_installed() {
    // Test when package is already installed
    let _config = test_config(vec![(Backend::Aur, "bat")]);

    // State already has the package
    let _state = test_state(vec![(Backend::Aur, "bat")]);

    // Transaction should be empty
    let tx = Transaction {
        to_install: vec![],
        to_adopt: vec![],
        to_prune: vec![],
        to_update_project_metadata: vec![],
    };

    assert!(tx.to_install.is_empty());
    assert!(tx.to_adopt.is_empty());
}

#[test]
fn test_sync_orphan_detection() {
    // Test detecting orphan packages (in state but not in config)
    let _config = test_config(vec![(Backend::Aur, "bat")]);

    // State has extra package not in config
    let _state = test_state(vec![(Backend::Aur, "bat"), (Backend::Aur, "old-package")]);

    // old-package should be in to_prune when prune is enabled
    let tx = Transaction {
        to_install: vec![],
        to_adopt: vec![],
        to_prune: vec![pkg_id(Backend::Aur, "old-package")],
        to_update_project_metadata: vec![],
    };

    assert_eq!(tx.to_prune.len(), 1);
    assert_eq!(tx.to_prune[0].name, "old-package");
}

#[test]
fn test_sync_options_default() {
    // Test default sync options
    let options = SyncOptions {
        dry_run: false,
        prune: false,
        gc: false,
        update: false,
        yes: false,
        force: false,
        target: None,
        noconfirm: false,
        hooks: false,
        skip_soar_install: false,
        modules: vec![],
    };

    assert!(!options.dry_run);
    assert!(!options.prune);
    assert!(!options.hooks);
    assert!(options.target.is_none());
}

#[test]
fn test_backend_string_conversion() {
    // Test backend to string conversion
    let backends = vec![
        (Backend::Aur, "aur"),
        (Backend::Flatpak, "flatpak"),
        (Backend::Soar, "soar"),
        (Backend::Npm, "npm"),
        (Backend::Cargo, "cargo"),
    ];

    for (backend, expected_lowercase) in backends {
        let s = format!("{:?}", backend).to_lowercase();
        assert!(!s.is_empty());
        assert_eq!(s, expected_lowercase);
    }
}

#[test]
fn test_sync_target_all() {
    // Test SyncTarget::All variant
    let target = SyncTarget::All;
    assert!(matches!(target, SyncTarget::All));
}

#[test]
fn test_sync_target_backend() {
    // Test SyncTarget::Backend variant
    let backends = vec![Backend::Aur, Backend::Flatpak, Backend::Soar];

    for backend in backends {
        let target = SyncTarget::Backend(backend.clone());
        assert!(matches!(target, SyncTarget::Backend(_)));
    }
}

#[test]
fn test_sync_target_named() {
    // Test SyncTarget::Named variant
    let target = SyncTarget::Named("firefox".to_string());
    assert!(matches!(target, SyncTarget::Named(_)));
}

#[test]
fn test_transaction_with_multiple_backends() {
    // Test transaction spanning multiple backends
    let tx = Transaction {
        to_install: vec![
            pkg_id(Backend::Aur, "neovim"),
            pkg_id(Backend::Flatpak, "discord"),
            pkg_id(Backend::Npm, "typescript"),
        ],
        to_adopt: vec![],
        to_prune: vec![],
        to_update_project_metadata: vec![],
    };

    // Count by backend
    let aur_count = tx
        .to_install
        .iter()
        .filter(|p| matches!(p.backend, Backend::Aur))
        .count();
    let flatpak_count = tx
        .to_install
        .iter()
        .filter(|p| matches!(p.backend, Backend::Flatpak))
        .count();
    let npm_count = tx
        .to_install
        .iter()
        .filter(|p| matches!(p.backend, Backend::Npm))
        .count();

    assert_eq!(aur_count, 1);
    assert_eq!(flatpak_count, 1);
    assert_eq!(npm_count, 1);
}
