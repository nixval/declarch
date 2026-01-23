use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a package manager backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend identifier (e.g., "aur", "npm", "pip")
    pub name: String,

    /// Binary name or path (e.g., "paru", "npm")
    /// Can be a list to try multiple alternatives: ["paru", "yay"]
    pub binary: BinarySpecifier,

    /// Command to list installed packages
    pub list_cmd: String,

    /// Command to install packages
    /// Use {packages} as placeholder for package list
    pub install_cmd: String,

    /// Command to remove packages
    /// Use {packages} as placeholder for package list
    pub remove_cmd: String,

    /// Optional: Command to query package info (for dependencies)
    pub query_cmd: Option<String>,

    /// How to parse the output of list_cmd
    pub list_format: OutputFormat,

    /// Column index for package name (for SplitWhitespace/TabSeparated)
    pub list_name_col: Option<usize>,

    /// Column index for package version (for SplitWhitespace/TabSeparated)
    pub list_version_col: Option<usize>,

    /// JSON path to packages array (for Json format)
    /// Empty string "" means root array
    pub list_json_path: Option<String>,

    /// Key name for package name in JSON objects
    pub list_name_key: Option<String>,

    /// Key name for package version in JSON objects
    pub list_version_key: Option<String>,

    /// Regex pattern to extract package info (for Regex format)
    pub list_regex: Option<String>,

    /// Capture group index for package name in regex
    pub list_regex_name_group: Option<usize>,

    /// Capture group index for package version in regex
    pub list_regex_version_group: Option<usize>,

    /// Flag to pass for auto-confirmation (e.g., "--noconfirm", "-y")
    pub noconfirm_flag: Option<String>,

    /// Whether command needs sudo
    pub needs_sudo: bool,

    /// Environment variables to set before running commands
    pub preinstall_env: Option<HashMap<String, String>>,

    /// Whether to use the Rust implementation instead of generic
    pub use_rust_fallback: bool,
}

/// Binary specifier - can be single or multiple alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinarySpecifier {
    Single(String),
    Multiple(Vec<String>),
}

impl BinarySpecifier {
    /// Get the first available binary
    pub fn find_available(&self) -> Option<String> {
        match self {
            BinarySpecifier::Single(bin) => which::which(bin).ok().map(|_| bin.clone()),
            BinarySpecifier::Multiple(binaries) => {
                for bin in binaries {
                    if which::which(bin).is_ok() {
                        return Some(bin.clone());
                    }
                }
                None
            }
        }
    }

    /// Get the primary binary name (for display/error messages)
    pub fn primary(&self) -> String {
        match self {
            BinarySpecifier::Single(bin) => bin.clone(),
            BinarySpecifier::Multiple(binaries) => binaries
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

/// Output format for list command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Space-separated columns: "package version"
    SplitWhitespace,

    /// Tab-separated columns
    TabSeparated,

    /// JSON output
    Json,

    /// Regex-based parsing
    Regex,

    /// Custom Rust parser (fallback to existing implementations)
    Custom,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            binary: BinarySpecifier::Single("unknown".to_string()),
            list_cmd: String::new(),
            install_cmd: String::new(),
            remove_cmd: String::new(),
            query_cmd: None,
            list_format: OutputFormat::SplitWhitespace,
            list_name_col: Some(0),
            list_version_col: Some(1),
            list_json_path: None,
            list_name_key: None,
            list_version_key: None,
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None,
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_specifier_single() {
        let bin = BinarySpecifier::Single("ls".to_string());
        assert_eq!(bin.primary(), "ls");
    }

    #[test]
    fn test_binary_specifier_multiple() {
        let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
        assert_eq!(bin.primary(), "paru");
    }

    #[test]
    fn test_backend_config_default() {
        let config = BackendConfig::default();
        assert_eq!(config.name, "unknown");
        assert_eq!(config.list_format, OutputFormat::SplitWhitespace);
    }
}
