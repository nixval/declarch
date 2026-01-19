use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Parse tab-separated output (like flatpak list --columns=...)
pub fn parse_tsv(
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

        let parts: Vec<&str> = line.split('\t').collect();

        if let Some(name) = parts.get(name_col) {
            let version = parts.get(version_col).map(|&v| v.to_string());

            installed.insert(name.to_string(), PackageMetadata {
                variant: None,
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
    fn test_parse_flatpak_output() {
        let output = "com.spotify.Client\t1.2.3\norg.mozilla.firefox\t120.0\n";
        let config = BackendConfig {
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = parse_tsv(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["com.spotify.Client"].version.as_deref(), Some("1.2.3"));
        assert_eq!(result["org.mozilla.firefox"].version.as_deref(), Some("120.0"));
    }

    #[test]
    fn test_parse_custom_columns() {
        // Format: "version\tpackage-name" (reversed)
        let output = "1.2.3\tcom.spotify.Client\n120.0\torg.mozilla.firefox\n";
        let config = BackendConfig {
            list_name_col: Some(1),
            list_version_col: Some(0),
            ..Default::default()
        };

        let result = parse_tsv(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["com.spotify.Client"].version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn test_parse_empty_lines() {
        let output = "com.spotify.Client\t1.2.3\n\norg.mozilla.firefox\t120.0\n";
        let config = BackendConfig {
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = parse_tsv(output, &config).unwrap();

        assert_eq!(result.len(), 2);
    }
}
