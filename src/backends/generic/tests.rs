use super::*;
use crate::backends::config::{BackendConfig, BinarySpecifier};
use crate::core::types::Backend;

#[test]
fn test_format_packages() {
    let config = BackendConfig {
        name: "test".to_string(),
        binary: BinarySpecifier::Single("echo".to_string()),
        fallback: None,
        ..Default::default()
    };

    let manager = GenericManager::from_config(config, Backend::from("aur"), false);
    let packages = vec!["pkg1".to_string(), "pkg2".to_string()];

    assert_eq!(manager.format_packages(&packages), "pkg1 pkg2");
}

#[test]
fn test_parse_search_regex_handles_alternative_capture_groups() {
    let config = BackendConfig {
        name: "test".to_string(),
        binary: BinarySpecifier::Single("echo".to_string()),
        search_regex: Some("^aur/([^\\s]+)|^community/([^\\s]+)|^extra/([^\\s]+)".to_string()),
        search_regex_name_group: Some(1),
        ..Default::default()
    };

    let manager = GenericManager::from_config(config, Backend::from("aur"), false);
    let stdout = "community/firefox 1.0\nextra/bat 2.0\naur/fd 3.0\n";
    let results = manager.parse_search_regex(stdout).unwrap();
    let names: Vec<_> = results.into_iter().map(|r| r.name).collect();
    assert_eq!(names, vec!["firefox", "bat", "fd"]);
}

#[test]
fn test_binary_specifier_primary() {
    let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
    assert_eq!(bin.primary(), "paru");
}

#[test]
fn test_sudo_applies_only_to_mutating_operations() {
    let config = BackendConfig {
        name: "test".to_string(),
        binary: BinarySpecifier::Single("sh".to_string()),
        needs_sudo: true,
        ..Default::default()
    };
    let manager = GenericManager::from_config(config, Backend::from("aur"), false);

    let read_cmd = manager
        .build_command("{binary} -c 'echo read'", CommandMode::ReadOnly)
        .expect("read command should build");
    let read_debug = format!("{:?}", read_cmd);
    assert!(
        !read_debug.contains("\"sudo\""),
        "read-only commands must not use sudo: {}",
        read_debug
    );

    let write_cmd = manager
        .build_command("{binary} -c 'echo write'", CommandMode::Mutating)
        .expect("write command should build");
    let write_debug = format!("{:?}", write_cmd);
    assert!(
        write_debug.contains("\"sudo\""),
        "mutating commands should use sudo when needs_sudo=true: {}",
        write_debug
    );
}
