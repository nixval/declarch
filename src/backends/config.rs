use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a package manager backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend identifier (e.g., "paru", "npm", "pip")
    pub name: String,

    /// Binary name or path (e.g., "paru", "npm")
    /// Can be a list to try multiple alternatives: ["paru", "yay"]
    pub binary: BinarySpecifier,

    /// Command to list installed packages
    /// Optional: if not set, backend cannot track installed packages (install-only mode)
    pub list_cmd: Option<String>,

    /// Command to install packages
    /// Use {packages} as placeholder for package list
    /// Required: backend must at least support install
    pub install_cmd: String,

    /// Command to remove packages
    /// Optional: if not set, packages cannot be removed via declarch
    pub remove_cmd: Option<String>,

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

    /// Optional package sources/repositories injected into command templates
    /// Use {repos} placeholder in command templates
    pub package_sources: Option<Vec<String>>,

    /// Whether to use the Rust implementation instead of generic
    pub use_rust_fallback: bool,

    /// ===== SEARCH SUPPORT =====
    /// Optional: Search command template
    /// Use {query} as placeholder for search term
    /// Example: "my-pm search {query} --json"
    pub search_cmd: Option<String>,

    /// How to parse the output of search_cmd
    pub search_format: Option<OutputFormat>,

    /// JSON path to results array (for Json format)
    pub search_json_path: Option<String>,

    /// Key name for package name in search results
    pub search_name_key: Option<String>,

    /// Key name for package version in search results
    pub search_version_key: Option<String>,

    /// Key name for package description in search results
    pub search_desc_key: Option<String>,

    /// Column index for package name in search results (for SplitWhitespace/TabSeparated)
    pub search_name_col: Option<usize>,

    /// Column index for package description in search results
    pub search_desc_col: Option<usize>,

    /// Regex pattern to extract search results
    pub search_regex: Option<String>,

    /// Capture group index for package name in search regex
    pub search_regex_name_group: Option<usize>,

    /// Capture group index for package description in search regex
    pub search_regex_desc_group: Option<usize>,

    /// ===== FALLBACK SUPPORT =====
    /// Optional fallback backend name if this backend is not available
    /// Example: paru → pacman, yarn → npm
    pub fallback: Option<String>,

    /// ===== UPDATE SUPPORT =====
    /// Optional: Command to update package list/index
    /// Example: "apt update", "pacman -Sy", "npm update"
    /// Use {binary} as placeholder for binary name
    pub update_cmd: Option<String>,

    /// ===== CACHE MANAGEMENT =====
    /// Optional: Command to clean package cache
    /// Example: "apt clean", "npm cache clean --force", "cargo cache --autoclean"
    /// Use {binary} as placeholder for binary name
    pub cache_clean_cmd: Option<String>,

    /// ===== UPGRADE SUPPORT =====
    /// Optional: Command to upgrade packages to latest version
    /// Example: "apt upgrade", "paru -Syu", "npm update -g"
    /// Use {binary} as placeholder for binary name
    /// Note: Some backends use {packages} for selective upgrade
    pub upgrade_cmd: Option<String>,

    /// ===== LOCAL SEARCH SUPPORT =====
    /// Optional: Command to search locally installed packages
    /// Example: "pacman -Q {query}", "dpkg -l {query}"
    /// Use {query} as placeholder for search term
    /// Use {binary} as placeholder for binary name
    pub search_local_cmd: Option<String>,

    /// How to parse the output of search_local_cmd
    pub search_local_format: Option<OutputFormat>,

    /// JSON path to results array for local search (for Json format)
    pub search_local_json_path: Option<String>,

    /// Key name for package name in local search results
    pub search_local_name_key: Option<String>,

    /// Key name for package version in local search results
    pub search_local_version_key: Option<String>,

    /// Column index for package name in local search results (for SplitWhitespace/TabSeparated)
    pub search_local_name_col: Option<usize>,

    /// Regex pattern to extract local search results
    pub search_local_regex: Option<String>,

    /// Capture group index for package name in local search regex
    pub search_local_regex_name_group: Option<usize>,
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

    /// JSON output (single JSON object/array)
    Json,

    /// JSON Lines (NDJSON) - each line is a separate JSON object
    JsonLines,

    /// NPM-style JSON - pseudo-array with inline commas
    /// Format: [\n{...}\n,\n{...}\n]
    NpmJson,

    /// JSON Object with keys as package names
    /// Format: {"pkg-name": {"version": "1.0"}, ...}
    /// The object keys are used as package names
    JsonObjectKeys,

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
            list_cmd: None,
            install_cmd: String::new(),
            remove_cmd: None,
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
            package_sources: None,
            use_rust_fallback: false,
            // Search support - all optional
            search_cmd: None,
            search_format: None,
            search_json_path: None,
            search_name_key: None,
            search_version_key: None,
            search_desc_key: None,
            search_name_col: None,
            search_desc_col: None,
            search_regex: None,
            search_regex_name_group: None,
            search_regex_desc_group: None,
            fallback: None,
            update_cmd: None,
            cache_clean_cmd: None,
            upgrade_cmd: None,
            search_local_cmd: None,
            search_local_format: None,
            search_local_json_path: None,
            search_local_name_key: None,
            search_local_version_key: None,
            search_local_name_col: None,
            search_local_regex: None,
            search_local_regex_name_group: None,
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
