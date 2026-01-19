use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Parse space-separated output (like pacman -Q)
///
/// Format: "package-name version"
pub fn parse_whitespace_split(
    output: &str,
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let mut installed = HashMap::new();
    let name_col = config.list_name_col.unwrap_or(0);
    let version_col = config.list_version_col.unwrap_or(1);

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if let Some(name) = parts.get(name_col) {
            let version = parts.get(version_col).map(|&v| v.to_string());

            installed.insert(name.to_string(), PackageMetadata {
                version,
                installed_at: Utc::now(),
                source_file: None,
            });
        }
    }

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::BackendConfig;

    #[test]
    fn test_parse_pacman_output() {
        let output = "pacman 6.0.2\nsystemd 255.1\n";
        let config = BackendConfig {
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = parse_whitespace_split(output, &config).unwrap();

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
            ..Default::default()
        };

        let result = parse_whitespace_split(output, &config).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_custom_columns() {
        // Format: "version package-name" (reversed)
        let output = "6.0.2 pacman\n255.1 systemd\n";
        let config = BackendConfig {
            list_name_col: Some(1),
            list_version_col: Some(0),
            ..Default::default()
        };

        let result = parse_whitespace_split(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["pacman"].version.as_deref(), Some("6.0.2"));
    }
}
