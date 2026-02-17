use super::*;
use crate::backends::config::BackendConfig;

fn setup() {
    clear_cache();
}

#[test]
fn test_parse_regex() {
    setup();

    let output = "pacman 6.0.2\nsystemd 255.1\n";
    let config = BackendConfig {
        list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
        list_regex_name_group: Some(1),
        list_regex_version_group: Some(2),
        fallback: None,
        ..Default::default()
    };

    let result = parse_regex(output, &config).expect("parse regex");

    assert_eq!(result.len(), 2);
    assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
    assert_eq!(result["systemd"].version.as_deref(), Some("255.1"));
}

#[test]
fn test_parse_brew_list() {
    setup();

    let output = "node@20 20.10.0\npython@3.12 3.12.1\n";
    let config = BackendConfig {
        list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
        list_regex_name_group: Some(1),
        list_regex_version_group: Some(2),
        fallback: None,
        ..Default::default()
    };

    let result = parse_regex(output, &config).expect("parse regex");

    assert_eq!(result.len(), 2);
    assert_eq!(result["node@20"].version.as_deref(), Some("20.10.0"));
}

#[test]
fn test_parse_custom_regex() {
    setup();

    let output = "Installing package-abc-1.0.0\nInstalling package-xyz-2.0.0\n";
    let config = BackendConfig {
        list_regex: Some(r"package-(\w+)-([\d.]+)".to_string()),
        list_regex_name_group: Some(1),
        list_regex_version_group: Some(2),
        fallback: None,
        ..Default::default()
    };

    let result = parse_regex(output, &config).expect("parse regex");

    assert_eq!(result.len(), 2);
    assert_eq!(result["abc"].version.as_deref(), Some("1.0.0"));
    assert_eq!(result["xyz"].version.as_deref(), Some("2.0.0"));
}

#[test]
fn test_regex_caching() {
    setup();

    let pattern = r"(\S+)\s+(\S+)";
    let config = BackendConfig {
        list_regex: Some(pattern.to_string()),
        list_regex_name_group: Some(1),
        list_regex_version_group: Some(2),
        fallback: None,
        ..Default::default()
    };

    let output1 = "pkg1 1.0\npkg2 2.0\n";
    let result1 = parse_regex(output1, &config).expect("first parse");
    assert_eq!(result1.len(), 2);

    let output2 = "pkg3 3.0\npkg4 4.0\n";
    let result2 = parse_regex(output2, &config).expect("second parse");
    assert_eq!(result2.len(), 2);

    assert!(regex_cache::is_cached(pattern));
}

#[test]
fn test_invalid_regex_error() {
    setup();

    let config = BackendConfig {
        list_regex: Some(r"[invalid(".to_string()),
        list_regex_name_group: Some(1),
        list_regex_version_group: Some(2),
        fallback: None,
        ..Default::default()
    };

    let result = parse_regex("test", &config);
    assert!(result.is_err());

    match result {
        Err(DeclarchError::ConfigError(_)) => (),
        _ => panic!("Expected ConfigError for invalid regex"),
    }
}
