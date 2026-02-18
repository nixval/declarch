use super::*;
use crate::backends::config::BackendConfig;

#[test]
fn test_parse_flatpak_output() {
    let output = "com.spotify.Client\t1.2.3\norg.mozilla.firefox\t120.0\n";
    let config = BackendConfig {
        list_name_col: Some(0),
        list_version_col: Some(1),
        fallback: None,
        ..Default::default()
    };

    let result = parse_tsv(output, &config).expect("parse tsv");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result["com.spotify.Client"].version.as_deref(),
        Some("1.2.3")
    );
    assert_eq!(
        result["org.mozilla.firefox"].version.as_deref(),
        Some("120.0")
    );
}

#[test]
fn test_parse_custom_columns() {
    let output = "1.2.3\tcom.spotify.Client\n120.0\torg.mozilla.firefox\n";
    let config = BackendConfig {
        list_name_col: Some(1),
        list_version_col: Some(0),
        fallback: None,
        ..Default::default()
    };

    let result = parse_tsv(output, &config).expect("parse tsv");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result["com.spotify.Client"].version.as_deref(),
        Some("1.2.3")
    );
}

#[test]
fn test_parse_empty_lines() {
    let output = "com.spotify.Client\t1.2.3\n\norg.mozilla.firefox\t120.0\n";
    let config = BackendConfig {
        list_name_col: Some(0),
        list_version_col: Some(1),
        fallback: None,
        ..Default::default()
    };

    let result = parse_tsv(output, &config).expect("parse tsv");

    assert_eq!(result.len(), 2);
}
