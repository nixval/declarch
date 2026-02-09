pub mod json_parser;
pub mod regex_parser;
pub mod tsv;
pub mod whitespace;

use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::Result;
use std::collections::HashMap;

/// Parse package list from command output
pub fn parse_package_list(
    output: &[u8],
    config: &BackendConfig,
) -> Result<HashMap<String, PackageMetadata>> {
    let stdout = String::from_utf8_lossy(output);

    match config.list_format {
        crate::backends::config::OutputFormat::SplitWhitespace => {
            whitespace::parse_whitespace_split(&stdout, config)
        }
        crate::backends::config::OutputFormat::TabSeparated => tsv::parse_tsv(&stdout, config),
        crate::backends::config::OutputFormat::Json => json_parser::parse_json(&stdout, config),
        crate::backends::config::OutputFormat::Regex => regex_parser::parse_regex(&stdout, config),
        crate::backends::config::OutputFormat::Custom => {
            // Custom parsers use Rust implementations
            Err(crate::error::DeclarchError::Other(
                "Custom format requires Rust implementation".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_whitespace() {
        let config = BackendConfig {
            list_name_col: Some(0),
            list_version_col: Some(1),
            fallback: None,
            ..Default::default()
        };

        let output = b"package1 1.0.0\npackage2 2.0.0\n";
        let result = parse_package_list(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("package1"));
        assert!(result.contains_key("package2"));
        assert_eq!(result["package1"].version.as_deref(), Some("1.0.0"));
    }
}
