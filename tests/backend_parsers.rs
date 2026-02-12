// Integration tests for backend output parsers
// Tests all parser formats: JSON, TSV, Whitespace, Regex, JSON Lines

use declarch::backends::config::{BackendConfig, OutputFormat};

mod json_tests {
    use super::*;

    #[test]
    fn test_json_nested_path() {
        let output = r#"{
            "data": {
                "installed": [
                    {"name": "curl", "version": "8.5.0"},
                    {"name": "wget", "version": "1.21.4"}
                ]
            }
        }"#;

        let config = BackendConfig {
            list_format: OutputFormat::Json,
            list_json_path: Some("data.installed".to_string()),
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["curl"].version.as_deref(), Some("8.5.0"));
        assert_eq!(result["wget"].version.as_deref(), Some("1.21.4"));
    }

    #[test]
    fn test_json_root_array() {
        let output = r#"[
            {"package": "rust", "ver": "1.75.0"},
            {"package": "cargo", "ver": "1.75.0"}
        ]"#;

        let config = BackendConfig {
            list_format: OutputFormat::Json,
            list_json_path: Some("".to_string()),
            list_name_key: Some("package".to_string()),
            list_version_key: Some("ver".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["rust"].version.as_deref(), Some("1.75.0"));
    }

    #[test]
    fn test_json_invalid() {
        let output = "not valid json";

        let config = BackendConfig {
            list_format: OutputFormat::Json,
            list_name_key: Some("name".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json(output, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_empty_array() {
        let output = r#"{"packages": []}"#;

        let config = BackendConfig {
            list_format: OutputFormat::Json,
            list_json_path: Some("packages".to_string()),
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json(output, &config).unwrap();
        assert!(result.is_empty());
    }
}

mod json_lines_tests {
    use super::*;

    #[test]
    fn test_json_lines_basic() {
        let output = r#"{"name": "pkg1", "version": "1.0.0"}
{"name": "pkg2", "version": "2.0.0"}
{"name": "pkg3", "version": "3.0.0"}"#;

        let config = BackendConfig {
            list_format: OutputFormat::JsonLines,
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json_lines(output, &config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result["pkg1"].version.as_deref(), Some("1.0.0"));
        assert_eq!(result["pkg2"].version.as_deref(), Some("2.0.0"));
        assert_eq!(result["pkg3"].version.as_deref(), Some("3.0.0"));
    }

    #[test]
    fn test_json_lines_with_invalid_lines() {
        // Some lines might be invalid JSON - should be skipped
        let output = r#"{"name": "valid", "version": "1.0"}
not valid json
{"name": "also_valid", "version": "2.0"}"#;

        let config = BackendConfig {
            list_format: OutputFormat::JsonLines,
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json_lines(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("valid"));
        assert!(result.contains_key("also_valid"));
    }

    #[test]
    fn test_json_lines_empty() {
        let output = "";

        let config = BackendConfig {
            list_format: OutputFormat::JsonLines,
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_json_lines(output, &config).unwrap();
        assert!(result.is_empty());
    }
}

mod whitespace_tests {
    use super::*;

    #[test]
    fn test_whitespace_basic() {
        let output = "vim 9.0\nneovim 0.9.0\nemacs 29.1";

        let config = BackendConfig {
            list_format: OutputFormat::SplitWhitespace,
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = declarch::backends::parsers::whitespace::parse_whitespace_split(output, &config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result["vim"].version.as_deref(), Some("9.0"));
        assert_eq!(result["neovim"].version.as_deref(), Some("0.9.0"));
        assert_eq!(result["emacs"].version.as_deref(), Some("29.1"));
    }

    #[test]
    fn test_whitespace_no_version() {
        let output = "package1\npackage2\npackage3";

        let config = BackendConfig {
            list_format: OutputFormat::SplitWhitespace,
            list_name_col: Some(0),
            list_version_col: None, // No version column
            ..Default::default()
        };

        let result = declarch::backends::parsers::whitespace::parse_whitespace_split(output, &config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result["package1"].version, None);
    }

    #[test]
    fn test_whitespace_empty_lines() {
        let output = "pkg1 1.0\n\npkg2 2.0\n   \npkg3 3.0";

        let config = BackendConfig {
            list_format: OutputFormat::SplitWhitespace,
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = declarch::backends::parsers::whitespace::parse_whitespace_split(output, &config).unwrap();

        assert_eq!(result.len(), 3);
    }
}

mod tsv_tests {
    use super::*;

    #[test]
    fn test_tsv_different_column_positions() {
        // Format: status\tname\tversion\tarch
        let output = "installed\tcurl\t8.5.0\tx86_64
installed\twget\t1.21.4\tx86_64
installed\tgit\t2.43.0\tx86_64";

        let config = BackendConfig {
            list_format: OutputFormat::TabSeparated,
            list_name_col: Some(1),    // 2nd column
            list_version_col: Some(2), // 3rd column
            ..Default::default()
        };

        let result = declarch::backends::parsers::tsv::parse_tsv(output, &config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result["curl"].version.as_deref(), Some("8.5.0"));
        assert_eq!(result["wget"].version.as_deref(), Some("1.21.4"));
    }

    #[test]
    fn test_tsv_short_lines() {
        // Lines with fewer columns - currently parser includes them with None version
        let output = "name1\tver1
name2
name3\tver3";

        let config = BackendConfig {
            list_format: OutputFormat::TabSeparated,
            list_name_col: Some(0),
            list_version_col: Some(1),
            ..Default::default()
        };

        let result = declarch::backends::parsers::tsv::parse_tsv(output, &config).unwrap();

        // Parser currently includes all lines, even those without version column
        assert_eq!(result.len(), 3);
        assert!(result.contains_key("name1"));
        assert!(result.contains_key("name2"));
        assert!(result.contains_key("name3"));
        // name2 should have no version
        assert_eq!(result["name2"].version, None);
    }
}

mod regex_tests {
    use super::*;

    #[test]
    fn test_regex_basic() {
        // Simulate pacman -Q output: "package version"
        // Note: parser uses captures_iter which processes the whole string
        let output = "vim 9.0.0-1
neovim 0.9.5-1
curl 8.5.0-1";

        let config = BackendConfig {
            list_format: OutputFormat::Regex,
            list_regex: Some(r"(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            ..Default::default()
        };

        let result = declarch::backends::parsers::regex_parser::parse_regex(output, &config).unwrap();

        // The regex should match all 3 lines
        assert_eq!(result.len(), 3);
        assert_eq!(result["vim"].version.as_deref(), Some("9.0.0-1"));
        assert_eq!(result["neovim"].version.as_deref(), Some("0.9.5-1"));
    }

    #[test]
    fn test_regex_multiline() {
        // Test with multiline flag to match line by line
        let output = "package1 1.0
package2 2.0
package3 3.0";

        let config = BackendConfig {
            list_format: OutputFormat::Regex,
            list_regex: Some(r"(?m)^(\S+)\s+(\S+)".to_string()),
            list_regex_name_group: Some(1),
            list_regex_version_group: Some(2),
            ..Default::default()
        };

        let result = declarch::backends::parsers::regex_parser::parse_regex(output, &config).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result["package1"].version.as_deref(), Some("1.0"));
        assert_eq!(result["package2"].version.as_deref(), Some("2.0"));
        assert_eq!(result["package3"].version.as_deref(), Some("3.0"));
    }

    #[test]
    fn test_regex_invalid_pattern() {
        let output = "some output";

        let config = BackendConfig {
            list_format: OutputFormat::Regex,
            list_regex: Some(r"[invalid(".to_string()), // Invalid regex
            list_regex_name_group: Some(1),
            ..Default::default()
        };

        let result = declarch::backends::parsers::regex_parser::parse_regex(output, &config);
        assert!(result.is_err());
    }
}

mod npm_json_tests {
    use super::*;

    #[test]
    fn test_npm_json_format() {
        // NPM search output format
        let output = r#"[
{"name": "lodash", "version": "4.17.21", "description": "JavaScript utility library"},
{"name": "express", "version": "4.18.2", "description": "Fast web framework"}
]"#;

        let config = BackendConfig {
            list_format: OutputFormat::NpmJson,
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            ..Default::default()
        };

        let result = declarch::backends::parsers::json_parser::parse_npm_json(output, &config).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["lodash"].version.as_deref(), Some("4.17.21"));
        assert_eq!(result["express"].version.as_deref(), Some("4.18.2"));
    }
}
