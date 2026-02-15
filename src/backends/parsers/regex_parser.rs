use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::{DeclarchError, Result};
use crate::utils::regex_cache;
use chrono::Utc;
use std::collections::HashMap;

/// Parse output using regex pattern
///
/// This function uses a global regex cache to avoid recompiling patterns
/// for repeated calls with the same backend configuration.
pub fn parse_regex(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let pattern = config.list_regex.as_ref().ok_or_else(|| {
        DeclarchError::ConfigError("Missing list_regex for regex parser".to_string())
    })?;

    let name_group = config.list_regex_name_group.unwrap_or(1);
    let version_group = config.list_regex_version_group.unwrap_or(2);

    let regex = regex_cache::get_cached_regex(pattern)
        .map_err(|e| DeclarchError::ConfigError(format!("Invalid regex pattern: {}", e)))?;

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

/// Clear the regex cache
///
/// This is mainly useful for testing to ensure a clean state
#[cfg(test)]
pub fn clear_cache() {
    regex_cache::clear_cache();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::BackendConfig;

    fn setup() {
        clear_cache();
    }

    #[test]
    fn test_parse_regex() {
        setup();

        // Pattern matches: "package-name version" with flexible spacing
        let output = "pacman 6.0.2\nsystemd 255.1\n";
        let config = BackendConfig {
            list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            fallback: None,
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
        assert_eq!(result["systemd"].version.as_deref(), Some("255.1"));
    }

    #[test]
    fn test_parse_brew_list() {
        setup();

        // brew list --versions format: "package 1.0 2.0 3.0"
        let output = "node@20 20.10.0\npython@3.12 3.12.1\n";
        let config = BackendConfig {
            list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            fallback: None,
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["node@20"].version.as_deref(), Some("20.10.0"));
    }

    #[test]
    fn test_parse_custom_regex() {
        setup();

        // Extract: "Installing package-name-1.0.0"
        let output = "Installing package-abc-1.0.0\nInstalling package-xyz-2.0.0\n";
        let config = BackendConfig {
            list_regex: Some(r"package-(\w+)-([\d.]+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            fallback: None,
            ..Default::default()
        };

        let result = parse_regex(output, &config).unwrap();

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

        // First call should compile and cache
        let output1 = "pkg1 1.0\npkg2 2.0\n";
        let result1 = parse_regex(output1, &config).unwrap();
        assert_eq!(result1.len(), 2);

        // Second call with same pattern should use cache
        let output2 = "pkg3 3.0\npkg4 4.0\n";
        let result2 = parse_regex(output2, &config).unwrap();
        assert_eq!(result2.len(), 2);

        // Verify cache has the pattern
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

        // Verify error is ConfigError (not Other)
        match result {
            Err(DeclarchError::ConfigError(_)) => (), // Expected
            _ => panic!("Expected ConfigError for invalid regex"),
        }
    }
}
