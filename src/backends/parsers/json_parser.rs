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
    let name_key = config.list_name_key.as_ref()
        .ok_or_else(|| DeclarchError::Other("Missing list_name_key for JSON parser".to_string()))?;

    let version_key = config.list_version_key.as_ref()
        .ok_or_else(|| DeclarchError::Other("Missing list_version_key for JSON parser".to_string()))?;

    let json: Value = serde_json::from_str(output)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse JSON: {}", e)))?;

    let mut installed = HashMap::new();

    // Navigate to the packages array using json_path
    let packages = match &config.list_json_path {
        Some(path) if !path.is_empty() => {
            // Navigate through JSON structure (e.g., "dependencies")
            navigate_json_path(&json, path)
        },
        _ => {
            // Root is the array
            Some(&json)
        }
    };

    let packages_array: Vec<&Value> = match packages {
        Some(Value::Array(arr)) => arr.iter().collect(),
        Some(Value::Object(obj)) => {
            // Handle case where root is object with package entries
            obj.values().collect()
        },
        _ => {
            return Ok(installed); // Empty or unexpected format
        }
    };

    for pkg in packages_array {
        if let Some(obj) = pkg.as_object() {
            if let Some(Value::String(name)) = obj.get(name_key) {
                let version = obj.get(version_key)
                    .and_then(|v: &Value| v.as_str())
                    .map(|v| v.to_string());

                installed.insert(name.to_string(), PackageMetadata {
                    version,
                    installed_at: Utc::now(),
                    source_file: None,
                });
            }
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
            },
            Value::Array(arr) => {
                let index = part.parse::<usize>().ok()?;
                current = arr.get(index)?;
            },
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
            ..Default::default()
        };

        let result = parse_json(output, &config).unwrap();

        assert_eq!(result.len(), 2);
    }
}
