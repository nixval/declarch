use super::*;
use crate::backends::config::BackendConfig;

#[test]
fn test_parse_pacman_output() {
    let output = "pacman 6.0.2\nsystemd 255.1\n";
    let config = BackendConfig {
        list_name_col: Some(0),
        list_version_col: Some(1),
        fallback: None,
        ..Default::default()
    };

    let result = parse_whitespace_split(output, &config).expect("parse whitespace");

    assert_eq!(result.len(), 2);
    assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
    assert_eq!(result["systemd"].version.as_deref(), Some("255.1"));
}

#[test]
fn test_parse_empty_lines() {
    let output = "pacman 6.0.2\n\nsystemd 255.1\n\n";
    let config = BackendConfig {
        list_name_col: Some(0),
        list_version_col: Some(1),
        fallback: None,
        ..Default::default()
    };

    let result = parse_whitespace_split(output, &config).expect("parse whitespace");

    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_custom_columns() {
    let output = "6.0.2 pacman\n255.1 systemd\n";
    let config = BackendConfig {
        list_name_col: Some(1),
        list_version_col: Some(0),
        fallback: None,
        ..Default::default()
    };

    let result = parse_whitespace_split(output, &config).expect("parse whitespace");

    assert_eq!(result.len(), 2);
    assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
}
