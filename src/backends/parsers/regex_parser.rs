use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::{DeclarchError, Result};
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;

/// Parse output using regex pattern
pub fn parse_regex(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let pattern = config
        .list_regex
        .as_ref()
        .ok_or_else(|| DeclarchError::Other("Missing list_regex for regex parser".to_string()))?;

    let name_group = config.list_regex_name_group.unwrap_or(1);
    let version_group = config.list_regex_version_group.unwrap_or(2);

    let regex = Regex::new(pattern)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let mut installed = HashMap::new();

    for caps in regex.captures_iter(output) {
        if let Some(name_match) = caps.get(name_group) {
            let name = name_match.as_str().to_string();
            let version = caps.get(version_group).map(|m| m.as_str().to_string());

            installed.insert(
                name,
                PackageMetadata {
                    variant: None,
                    version,
                    installed_at: Utc::now(),
                    source_file: None,
                },
            );
        }
    }

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::BackendConfig;

    #[test]
    fn test_parse_regex() {
        // Pattern matches: "package-name version" with flexible spacing
        let output = "pacman 6.0.2\nsystemd 255.1\n";
        let config = BackendConfig {
            list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
        assert_eq!(result["systemd"].version.as_deref(), Some("255.1"));
    }

    #[test]
    fn test_parse_brew_list() {
        // brew list --versions format: "package 1.0 2.0 3.0"
        let output = "node@20 20.10.0\npython@3.12 3.12.1\n";
        let config = BackendConfig {
            list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["node@20"].version.as_deref(), Some("20.10.0"));
    }

    #[test]
    fn test_parse_custom_regex() {
        // Extract: "Installing package-name-1.0.0"
        let output = "Installing package-abc-1.0.0\nInstalling package-xyz-2.0.0\n";
        let config = BackendConfig {
            list_regex: Some(r"package-(\w+)-([\d.]+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["abc"].version.as_deref(), Some("1.0.0"));
        assert_eq!(result["xyz"].version.as_deref(), Some("2.0.0"));
    }
}
