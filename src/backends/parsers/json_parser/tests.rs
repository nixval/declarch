
use super::*;
use crate::backends::config::BackendConfig;

#[test]
fn test_parse_npm_json() {
    let output = r#"[
            {"name": "npm", "version": "10.0.0"},
            {"name": "pnpm", "version": "8.0.0"}
        ]"#;

    let config = BackendConfig {
        list_json_path: Some("".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result["npm"].version.as_deref(), Some("10.0.0"));
    assert_eq!(result["pnpm"].version.as_deref(), Some("8.0.0"));
}

#[test]
fn test_parse_nested_json() {
    let output = r#"{
            "dependencies": {
                "npm": {"name": "npm", "version": "10.0.0"},
                "pnpm": {"name": "pnpm", "version": "8.0.0"}
            }
        }"#;

    let config = BackendConfig {
        list_json_path: Some("dependencies".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_pip_json() {
    let output = r#"[
            {"name": "pip", "version": "23.0"},
            {"name": "setuptools", "version": "69.0"}
        ]"#;

    let config = BackendConfig {
        list_json_path: Some("".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_npm_object_format() {
    // npm list -g --depth=0 --json output format
    // Key is package name, value contains metadata (no "name" field in object!)
    let output = r#"{
            "dependencies": {
                "npm-stat": {
                    "version": "0.1.0",
                    "overridden": false
                },
                "npms": {
                    "version": "0.3.2",
                    "overridden": false
                }
            }
        }"#;

    let config = BackendConfig {
        list_json_path: Some("dependencies".to_string()),
        list_name_key: Some("name".to_string()), // Not used for Object format
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json(output, &config).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get("npm-stat").unwrap().version.as_deref(),
        Some("0.1.0")
    );
    assert_eq!(
        result.get("npms").unwrap().version.as_deref(),
        Some("0.3.2")
    );
}

#[test]
fn test_parse_json_object_keys() {
    // npm list -g --json format
    let output = r#"{
            "version": "10.2.3",
            "dependencies": {
                "npm": {
                    "version": "10.2.3",
                    "resolved": "https://registry.npmjs.org/npm/-/npm-10.2.3.tgz"
                },
                "typescript": {
                    "version": "5.3.2"
                },
                "pnpm": {
                    "version": "8.10.0"
                }
            }
        }"#;

    let config = BackendConfig {
        list_json_path: Some("dependencies".to_string()),
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json_object_keys(output, &config).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result["npm"].version.as_deref(), Some("10.2.3"));
    assert_eq!(result["typescript"].version.as_deref(), Some("5.3.2"));
    assert_eq!(result["pnpm"].version.as_deref(), Some("8.10.0"));
}

#[test]
fn test_parse_json_object_keys_empty() {
    let output = r#"{"dependencies": {}}"#;

    let config = BackendConfig {
        list_json_path: Some("dependencies".to_string()),
        list_version_key: Some("version".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = parse_json_object_keys(output, &config).unwrap();
    assert!(result.is_empty());
}
