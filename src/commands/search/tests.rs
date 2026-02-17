use super::*;
use crate::backends::config::BackendConfig;
use crate::state::types::{PackageState, State};
use chrono::Utc;

#[test]
fn select_backends_filters_unknown_and_unsupported() {
    let mut all = HashMap::new();
    all.insert(
        "paru".to_string(),
        BackendConfig {
            name: "paru".to_string(),
            search_cmd: Some("paru -Ss {query}".to_string()),
            ..Default::default()
        },
    );
    all.insert(
        "pip".to_string(),
        BackendConfig {
            name: "pip".to_string(),
            search_local_cmd: Some("pip list | grep {query}".to_string()),
            ..Default::default()
        },
    );

    let requested = vec!["paru".to_string(), "pip".to_string(), "missing".to_string()];
    let (selected_remote, unknown_remote, unsupported_remote, os_mismatch_remote) =
        select_backends_to_search(&all, Some(&requested), false);
    let names_remote: Vec<_> = selected_remote
        .iter()
        .map(|b| b.name().to_string())
        .collect();
    assert_eq!(names_remote, vec!["paru".to_string()]);
    assert_eq!(unknown_remote, vec!["missing".to_string()]);
    assert_eq!(unsupported_remote, vec!["pip".to_string()]);
    assert!(os_mismatch_remote.is_empty());

    let (selected_local, unknown_local, unsupported_local, os_mismatch_local) =
        select_backends_to_search(&all, Some(&requested), true);
    let names_local: Vec<_> = selected_local
        .iter()
        .map(|b| b.name().to_string())
        .collect();
    assert_eq!(names_local, vec!["pip".to_string()]);
    assert_eq!(unknown_local, vec!["missing".to_string()]);
    assert_eq!(unsupported_local, vec!["paru".to_string()]);
    assert!(os_mismatch_local.is_empty());
}

#[test]
fn select_backends_auto_mode_is_sorted() {
    let mut all = HashMap::new();
    all.insert(
        "zypper".to_string(),
        BackendConfig {
            name: "zypper".to_string(),
            search_cmd: Some("zypper se {query}".to_string()),
            ..Default::default()
        },
    );
    all.insert(
        "apt".to_string(),
        BackendConfig {
            name: "apt".to_string(),
            search_cmd: Some("apt search {query}".to_string()),
            ..Default::default()
        },
    );

    let (selected, unknown, unsupported, os_mismatch) =
        select_backends_to_search(&all, None, false);
    let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
    assert_eq!(names, vec!["apt".to_string(), "zypper".to_string()]);
    assert!(unknown.is_empty());
    assert!(unsupported.is_empty());
    assert!(os_mismatch.is_empty());
}

#[test]
fn select_backends_local_mode_accepts_list_fallback() {
    let mut all = HashMap::new();
    all.insert(
        "flatpak".to_string(),
        BackendConfig {
            name: "flatpak".to_string(),
            list_cmd: Some("flatpak list".to_string()),
            ..Default::default()
        },
    );

    let (selected, unknown, unsupported, os_mismatch) = select_backends_to_search(&all, None, true);
    let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
    assert_eq!(names, vec!["flatpak".to_string()]);
    assert!(unknown.is_empty());
    assert!(unsupported.is_empty());
    assert!(os_mismatch.is_empty());
}

#[test]
fn select_backends_local_mode_deduplicates_arch_family() {
    let mut all = HashMap::new();
    all.insert(
        "aur".to_string(),
        BackendConfig {
            name: "aur".to_string(),
            search_local_cmd: Some("aur-local {query}".to_string()),
            ..Default::default()
        },
    );
    all.insert(
        "yay".to_string(),
        BackendConfig {
            name: "yay".to_string(),
            search_local_cmd: Some("yay-local {query}".to_string()),
            ..Default::default()
        },
    );
    all.insert(
        "flatpak".to_string(),
        BackendConfig {
            name: "flatpak".to_string(),
            search_local_cmd: Some("flatpak search --columns=application {query}".to_string()),
            ..Default::default()
        },
    );

    let (selected, unknown, unsupported, os_mismatch) = select_backends_to_search(&all, None, true);
    let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
    assert_eq!(names, vec!["aur".to_string(), "flatpak".to_string()]);
    assert!(unknown.is_empty());
    assert!(unsupported.is_empty());
    assert!(os_mismatch.is_empty());
}

#[test]
fn normalize_package_name_strips_repo_prefix() {
    assert_eq!(normalize_package_name("bat"), "bat");
    assert_eq!(normalize_package_name("extra/bat"), "bat");
}

#[test]
fn canonical_backend_group_maps_arch_family() {
    assert_eq!(canonical_backend_group("aur"), "arch");
    assert_eq!(canonical_backend_group("paru"), "arch");
    assert_eq!(canonical_backend_group("yay"), "arch");
    assert_eq!(canonical_backend_group("pacman"), "arch");
    assert_eq!(canonical_backend_group("flatpak"), "flatpak");
}

#[test]
fn fallback_local_errors_are_suppressed_without_verbose() {
    assert!(!should_show_backend_error(
        "Local list fallback failed: Package manager error: xyz",
        false,
        true
    ));
    assert!(!should_show_backend_error(
        "Local search failed: xyz",
        false,
        true
    ));
    assert!(should_show_backend_error(
        "Search failed: xyz",
        false,
        false
    ));
    assert!(should_show_backend_error(
        "Local list fallback failed: Package manager error: xyz",
        true,
        true
    ));
}

#[test]
fn selection_warnings_are_quiet_in_auto_mode_without_verbose() {
    let auto_quiet = SearchOptions {
        query: "bat".to_string(),
        backends: None,
        limit: None,
        installed_only: false,
        available_only: false,
        local: false,
        verbose: false,
        format: None,
        output_version: None,
    };
    assert!(!should_emit_selection_warning(&auto_quiet));

    let explicit_backend = SearchOptions {
        backends: Some(vec!["aur".to_string()]),
        ..auto_quiet
    };
    assert!(should_emit_selection_warning(&explicit_backend));
}

#[test]
fn installed_match_handles_repo_prefix_and_backend_alias() {
    let mut state = State::default();
    state.packages.insert(
        "aur:bat".to_string(),
        PackageState {
            backend: Backend::from("aur"),
            config_name: "bat".to_string(),
            provides_name: "bat".to_string(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: Some("0.25.0".to_string()),
            install_reason: Some("declared".to_string()),
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );

    let result = PackageSearchResult {
        name: "extra/bat".to_string(),
        version: Some("0.25.0".to_string()),
        description: None,
        backend: Backend::from("pacman"),
    };

    assert!(is_installed_result(&result, &state, false));
}

#[test]
fn collect_managed_hits_respects_backend_filter() {
    let mut state = State::default();
    state.packages.insert(
        "aur:bat".to_string(),
        PackageState {
            backend: Backend::from("aur"),
            config_name: "bat".to_string(),
            provides_name: "bat".to_string(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: Some("0.25.0".to_string()),
            install_reason: Some("declared".to_string()),
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );
    state.packages.insert(
        "brew:hello".to_string(),
        PackageState {
            backend: Backend::from("brew"),
            config_name: "hello".to_string(),
            provides_name: "hello".to_string(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: Some("2.12.2".to_string()),
            install_reason: Some("declared".to_string()),
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        },
    );

    let options = SearchOptions {
        query: "hello".to_string(),
        backends: Some(vec!["brew".to_string()]),
        limit: None,
        installed_only: false,
        available_only: false,
        local: false,
        verbose: false,
        format: None,
        output_version: None,
    };

    let grouped = collect_managed_hits("hello", &state, &options);
    assert_eq!(grouped.len(), 1);
    assert!(grouped.contains_key("brew"));
}
