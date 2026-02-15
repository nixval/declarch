//! JSON parsers for backend outputs
//!
//! Supports various JSON formats used by package managers.

use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::{DeclarchError, Result};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

/// Parse JSON output (like npm list --json)
pub fn parse_json(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let name_key = config
        .list_name_key
        .as_ref()
        .ok_or_else(|| DeclarchError::Other("Missing list_name_key for JSON parser".to_string()))?;

    let version_key = config.list_version_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_version_key for JSON parser".to_string())
    })?;

    let json: Value = serde_json::from_str(output)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse JSON: {}", e)))?;

    let mut installed = HashMap::new();

    // Navigate to the packages array using json_path
    let packages = match &config.list_json_path {
        Some(path) if !path.is_empty() => {
            // Navigate through JSON structure (e.g., "dependencies")
            navigate_json_path(&json, path)
        }
        _ => {
            // Root is the array
            Some(&json)
        }
    };

    // Handle both Array and Object structures for packages
    if let Some(packages) = packages {
        match packages {
            Value::Array(arr) => {
                // Array format: [{"name": "pkg", "version": "1.0"}, ...]
                for pkg in arr.iter() {
                    if let Some(obj) = pkg.as_object()
                        && let Some(Value::String(name)) = obj.get(name_key)
                    {
                        let version = obj
                            .get(version_key)
                            .and_then(|v: &Value| v.as_str())
                            .map(|v| v.to_string());

                        installed.insert(
                            name.to_string(),
                            PackageMetadata {
                                version,
                                variant: None,
                                installed_at: Utc::now(),
                                source_file: None,
                            },
                        );
                    }
                }
            }
            Value::Object(obj) => {
                // Object format: {"pkg-name": {"version": "1.0"}, ...}
                // Key is package name, value contains metadata
                for (name, metadata) in obj.iter() {
                    if let Some(metadata_obj) = metadata.as_object() {
                        let version = metadata_obj
                            .get(version_key)
                            .and_then(|v: &Value| v.as_str())
                            .map(|v| v.to_string());

                        installed.insert(
                            name.to_string(),
                            PackageMetadata {
                                version,
                                variant: None,
                                installed_at: Utc::now(),
                                source_file: None,
                            },
                        );
                    }
                }
            }
            _ => {} // Empty or unexpected format
        }
    }

    Ok(installed)
}

/// Parse JSON Lines (NDJSON) output - each line is a separate JSON object
/// Used by tools like soar, docker, etc.
pub fn parse_json_lines(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let name_key = config.list_name_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_name_key for JSON Lines parser".to_string())
    })?;

    let version_key = config.list_version_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_version_key for JSON Lines parser".to_string())
    })?;

    let mut installed = HashMap::new();

    // Parse each line as a separate JSON object
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to parse the line as JSON
        match serde_json::from_str::<Value>(line) {
            Ok(json) => {
                // Extract package name
                if let Some(Value::String(name)) = json.get(name_key) {
                    let version = json
                        .get(version_key)
                        .and_then(|v: &Value| v.as_str())
                        .map(|v| v.to_string());

                    installed.insert(
                        name.to_string(),
                        PackageMetadata {
                            version,
                            variant: None,
                            installed_at: Utc::now(),
                            source_file: None,
                        },
                    );
                }
            }
            Err(_) => {
                // Skip lines that aren't valid JSON (like log messages, tables, etc.)
                continue;
            }
        }
    }

    Ok(installed)
}

/// Parse NPM-style JSON output
/// Format: [
///   {"name": "pkg1", ...}
///   ,
///   {"name": "pkg2", ...}
/// ]
pub fn parse_npm_json(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let name_key = config.list_name_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_name_key for NPM JSON parser".to_string())
    })?;

    let version_key = config.list_version_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_version_key for NPM JSON parser".to_string())
    })?;

    let mut installed = HashMap::new();

    // NPM output format is: [\n{...}\n,\n{...}\n]
    // Each line can be: "[", "{...}", ",", or "]"
    for line in output.lines() {
        let line = line.trim();

        // Skip array markers and commas
        if line.is_empty() || line == "[" || line == "]" || line == "," {
            continue;
        }

        // Lines might end with comma, remove it
        let line = line.trim_end_matches(',');

        // Try to parse as JSON object
        match serde_json::from_str::<Value>(line) {
            Ok(json) => {
                if let Some(Value::String(name)) = json.get(name_key) {
                    let version = json
                        .get(version_key)
                        .and_then(|v: &Value| v.as_str())
                        .map(|v| v.to_string());

                    installed.insert(
                        name.to_string(),
                        PackageMetadata {
                            version,
                            variant: None,
                            installed_at: Utc::now(),
                            source_file: None,
                        },
                    );
                }
            }
            Err(_) => {
                // Skip non-JSON lines
                continue;
            }
        }
    }

    Ok(installed)
}

/// Parse JSON Object with keys as package names
/// Format: {"pkg-name": {"version": "1.0"}, ...}
/// Used by npm list -g --json, pip list --format=json, etc.
pub fn parse_json_object_keys(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let version_key = config.list_version_key.as_ref().ok_or_else(|| {
        DeclarchError::Other("Missing list_version_key for JsonObjectKeys parser".to_string())
    })?;

    let json: Value = serde_json::from_str(output)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse JSON: {}", e)))?;

    let mut installed = HashMap::new();

    // Navigate to the packages object using json_path
    let packages = match &config.list_json_path {
        Some(path) if !path.is_empty() => {
            // Navigate through JSON structure (e.g., "dependencies")
            navigate_json_path(&json, path)
        }
        _ => {
            // Root is the object
            Some(&json)
        }
    };

    // Parse object where keys are package names
    if let Some(Value::Object(obj)) = packages {
        for (name, metadata) in obj.iter() {
            // Skip special npm keys
            if name.starts_with("_") || name == "version" {
                continue;
            }

            let version = if let Some(metadata_obj) = metadata.as_object() {
                metadata_obj
                    .get(version_key)
                    .and_then(|v: &Value| v.as_str())
                    .map(|v| v.to_string())
            } else {
                None
            };

            installed.insert(
                name.to_string(),
                PackageMetadata {
                    version,
                    variant: None,
                    installed_at: Utc::now(),
                    source_file: None,
                },
            );
        }
    }

    Ok(installed)
}

/// Navigate through JSON structure using dot notation path
fn navigate_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                let index = part.parse::<usize>().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
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
}
