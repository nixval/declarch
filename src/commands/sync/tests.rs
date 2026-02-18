use super::*;
use crate::backends::config::BackendConfig;
use crate::core::types::PackageId;
use std::collections::HashMap;
use std::path::PathBuf;

fn merged_config_with_options(
    backend_name: &str,
    options: &[(&str, &str)],
) -> loader::MergedConfig {
    let mut merged = loader::MergedConfig::default();
    let mut backend_opts = HashMap::new();
    for (k, v) in options {
        backend_opts.insert((*k).to_string(), (*v).to_string());
    }
    merged
        .backend_options
        .insert(backend_name.to_string(), backend_opts);
    merged
}

#[test]
fn test_backend_option_overrides_disable_sentinel() {
    let mut backend = BackendConfig {
        name: "paru".to_string(),
        install_cmd: "paru -S {packages}".to_string(),
        remove_cmd: Some("paru -R {packages}".to_string()),
        list_cmd: Some("paru -Q".to_string()),
        search_cmd: Some("paru -Ss {query}".to_string()),
        search_local_cmd: Some("paru -Q {query}".to_string()),
        update_cmd: Some("paru -Sy".to_string()),
        cache_clean_cmd: Some("paru -Sc".to_string()),
        upgrade_cmd: Some("paru -Syu".to_string()),
        fallback: Some("pacman".to_string()),
        noconfirm_flag: Some("--noconfirm".to_string()),
        needs_sudo: true,
        ..Default::default()
    };

    let merged = merged_config_with_options(
        "paru",
        &[
            ("install_cmd", "-"),
            ("remove_cmd", "-"),
            ("list_cmd", "-"),
            ("search_cmd", "-"),
            ("search_local_cmd", "-"),
            ("update_cmd", "-"),
            ("cache_clean_cmd", "-"),
            ("upgrade_cmd", "-"),
            ("fallback", "-"),
            ("noconfirm_flag", "-"),
            ("needs_sudo", "invalid"),
            ("unknown_key", "value"),
        ],
    );

    apply_backend_option_overrides(&mut backend, "paru", &merged);

    assert_eq!(backend.install_cmd, "paru -S {packages}");
    assert!(backend.remove_cmd.is_none());
    assert!(backend.list_cmd.is_none());
    assert!(backend.search_cmd.is_none());
    assert!(backend.search_local_cmd.is_none());
    assert!(backend.update_cmd.is_none());
    assert!(backend.cache_clean_cmd.is_none());
    assert!(backend.upgrade_cmd.is_none());
    assert!(backend.fallback.is_none());
    assert!(backend.noconfirm_flag.is_none());
    assert!(backend.needs_sudo);
}

#[test]
fn test_backend_option_overrides_apply_valid_values() {
    let mut backend = BackendConfig {
        name: "pacman".to_string(),
        install_cmd: "pacman -S {packages}".to_string(),
        remove_cmd: Some("pacman -R {packages}".to_string()),
        needs_sudo: false,
        ..Default::default()
    };

    let merged = merged_config_with_options(
        "pacman",
        &[
            ("remove_cmd", "pacman -Rns {packages}"),
            ("update_cmd", "pacman -Sy"),
            ("needs_sudo", "on"),
            ("noconfirm_flag", "--noconfirm"),
        ],
    );

    apply_backend_option_overrides(&mut backend, "pacman", &merged);

    assert_eq!(
        backend.remove_cmd.as_deref(),
        Some("pacman -Rns {packages}")
    );
    assert_eq!(backend.update_cmd.as_deref(), Some("pacman -Sy"));
    assert_eq!(backend.noconfirm_flag.as_deref(), Some("--noconfirm"));
    assert!(backend.needs_sudo);
}

#[test]
fn test_backend_option_overrides_reject_invalid_templates() {
    let mut backend = BackendConfig {
        name: "paru".to_string(),
        install_cmd: "paru -S {packages}".to_string(),
        remove_cmd: Some("paru -R {packages}".to_string()),
        search_cmd: Some("paru -Ss {query}".to_string()),
        search_local_cmd: Some("paru -Q {query}".to_string()),
        ..Default::default()
    };

    let merged = merged_config_with_options(
        "paru",
        &[
            ("install_cmd", "paru -S"),
            ("remove_cmd", "paru -R"),
            ("search_cmd", "paru -Ss"),
            ("search_local_cmd", "paru -Q"),
        ],
    );

    apply_backend_option_overrides(&mut backend, "paru", &merged);

    assert_eq!(backend.install_cmd, "paru -S {packages}");
    assert_eq!(backend.remove_cmd.as_deref(), Some("paru -R {packages}"));
    assert_eq!(backend.search_cmd.as_deref(), Some("paru -Ss {query}"));
    assert_eq!(backend.search_local_cmd.as_deref(), Some("paru -Q {query}"));
}

#[test]
fn test_resolve_target_detects_backend_from_imports() {
    let mut merged = loader::MergedConfig::default();
    merged.backends.push(BackendConfig {
        name: "paru".to_string(),
        ..Default::default()
    });

    let target = resolve_target(&Some("paru".to_string()), &merged);
    match target {
        SyncTarget::Backend(b) => assert_eq!(b.name(), "paru"),
        _ => panic!("expected backend target"),
    }
}

#[test]
fn test_named_target_exists_by_package_or_module_stem() {
    let mut merged = loader::MergedConfig::default();
    let pkg = PackageId {
        name: "bat".to_string(),
        backend: Backend::from("paru"),
    };
    merged
        .packages
        .insert(pkg, vec![PathBuf::from("/tmp/devtools.kdl")]);

    assert!(named_target_exists(&merged, "bat"));
    assert!(named_target_exists(&merged, "devtools"));
    assert!(!named_target_exists(&merged, "unknown"));
}

#[test]
fn test_apply_backend_package_sources_normalizes_and_dedupes() {
    let mut backend = BackendConfig {
        name: "paru".to_string(),
        package_sources: Some(vec!["core".to_string(), " extra ".to_string()]),
        ..Default::default()
    };
    let mut merged = loader::MergedConfig::default();
    merged.package_sources.insert(
        "paru".to_string(),
        vec![
            "extra".to_string(),
            "multilib".to_string(),
            "   ".to_string(),
            "core".to_string(),
        ],
    );

    apply_backend_package_sources(&mut backend, "paru", &merged);

    assert_eq!(
        backend.package_sources,
        Some(vec![
            "core".to_string(),
            "extra".to_string(),
            "multilib".to_string()
        ])
    );
}
