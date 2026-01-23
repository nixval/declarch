// Integration tests for custom backend configurations
// Test NALA, ZYPPER, DNF5 parsers

use declarch::backends::config::{BackendConfig, OutputFormat};

#[test]
fn test_nala_json_parser() {
    let output = r#"{
  "packages": [
    {"name": "vim", "version": "2:9.0.1000-0ubuntu1"},
    {"name": "neovim", "version": "0.9.1-1"},
    {"name": "git", "version": "1:2.43.0-0ubuntu1"}
  ]
}"#;

    let config = BackendConfig {
        list_format: OutputFormat::Json,
        list_json_path: Some("packages".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        ..Default::default()
    };

    let result = declarch::backends::parsers::json_parser::parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(
        result["vim"].version.as_deref(),
        Some("2:9.0.1000-0ubuntu1")
    );
    assert_eq!(result["neovim"].version.as_deref(), Some("0.9.1-1"));
    assert_eq!(result["git"].version.as_deref(), Some("1:2.43.0-0ubuntu1"));
}

#[test]
fn test_zypper_tsv_parser() {
    // Zypper output is tab-separated with | as visual separators
    // Format: status | repo | name | version | arch
    let output = "i+\tMain\tfirefox\t120.0-1.1\tx86_64
i+\tMain\tpython3\t3.11.6-1.1\tx86_64
i+\tMain\tgcc\t13.2.0-1.1\tx86_64";

    let config = BackendConfig {
        list_format: OutputFormat::TabSeparated,
        list_name_col: Some(2),    // 3rd column (0-indexed)
        list_version_col: Some(3), // 4th column
        ..Default::default()
    };

    let result = declarch::backends::parsers::tsv::parse_tsv(output, &config).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result["firefox"].version.as_deref(), Some("120.0-1.1"));
    assert_eq!(result["python3"].version.as_deref(), Some("3.11.6-1.1"));
    assert_eq!(result["gcc"].version.as_deref(), Some("13.2.0-1.1"));
}

#[test]
fn test_dnf5_json_parser() {
    let output = r#"{
  "packages": [
    {"name": "neovim", "version": "0.9.5-1.fc39"},
    {"name": "podman", "version": "4.9.4-1.fc39"},
    {"name": "toolbox", "version": "0.0.99.5-1.fc39"}
  ]
}"#;

    let config = BackendConfig {
        list_format: OutputFormat::Json,
        list_json_path: Some("packages".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        ..Default::default()
    };

    let result = declarch::backends::parsers::json_parser::parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result["neovim"].version.as_deref(), Some("0.9.5-1.fc39"));
    assert_eq!(result["podman"].version.as_deref(), Some("4.9.4-1.fc39"));
    assert_eq!(
        result["toolbox"].version.as_deref(),
        Some("0.0.99.5-1.fc39")
    );
}
