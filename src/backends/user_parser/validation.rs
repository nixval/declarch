use crate::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
use crate::error::{DeclarchError, Result};
use crate::ui;

/// Check if backend has multiple binary alternatives (needs {binary} placeholder)
fn has_multiple_binaries(config: &BackendConfig) -> bool {
    matches!(config.binary, BinarySpecifier::Multiple(_))
}

/// Validate backend configuration
pub(super) fn validate_backend_config(config: &BackendConfig) -> Result<()> {
    // install_cmd is required - backend must at least support install
    if config.install_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'install_cmd' cannot be empty".to_string(),
        ));
    }

    // Check if this backend has multiple binary alternatives
    let needs_binary_placeholder = has_multiple_binaries(config);

    // Validate required placeholders in commands
    // This prevents runtime errors due to typos in backend definitions

    // list_cmd is optional but recommended
    if let Some(ref list_cmd) = config.list_cmd {
        // Only warn about {binary} if backend has multiple binaries
        if needs_binary_placeholder && !list_cmd.contains("{binary}") {
            ui::warning(&format!(
                "Backend '{}' has multiple binaries but list_cmd missing '{{binary}}' placeholder",
                config.name
            ));
        }
    } else {
        // Keep non-verbose output clean; show install-only diagnostics in verbose mode.
        ui::verbose(&format!(
            "Backend '{}' install-only mode (no list_cmd configured)",
            config.name
        ));
    }

    // install_cmd should contain {packages} placeholder
    if !config.install_cmd.contains("{packages}") {
        return Err(DeclarchError::ConfigError(format!(
            "Backend '{}' install_cmd must contain '{{packages}}' placeholder",
            config.name
        )));
    }

    // remove_cmd is optional
    if let Some(ref remove_cmd) = config.remove_cmd {
        // remove_cmd should contain {packages} placeholder
        if !remove_cmd.contains("{packages}") {
            return Err(DeclarchError::ConfigError(format!(
                "Backend '{}' remove_cmd must contain '{{packages}}' placeholder",
                config.name
            )));
        }
    }
    // Note: remove_cmd being None is normal (install-only backends), no warning needed

    // search_cmd should contain {binary} and {query} if configured
    if let Some(ref search_cmd) = config.search_cmd {
        if needs_binary_placeholder && !search_cmd.contains("{binary}") {
            ui::warning(&format!(
                "Backend '{}' has multiple binaries but search_cmd missing '{{binary}}' placeholder",
                config.name
            ));
        }
        if !search_cmd.contains("{query}") {
            return Err(DeclarchError::ConfigError(format!(
                "Backend '{}' search_cmd must contain '{{query}}' placeholder",
                config.name
            )));
        }
    }

    // update_cmd should contain {binary} if backend has multiple binaries
    if let Some(ref update_cmd) = config.update_cmd
        && needs_binary_placeholder
        && !update_cmd.contains("{binary}")
    {
        ui::warning(&format!(
            "Backend '{}' has multiple binaries but update_cmd missing '{{binary}}' placeholder",
            config.name
        ));
    }

    // cache_clean_cmd should contain {binary} if backend has multiple binaries
    if let Some(ref cache_clean_cmd) = config.cache_clean_cmd
        && needs_binary_placeholder
        && !cache_clean_cmd.contains("{binary}")
    {
        ui::warning(&format!(
            "Backend '{}' has multiple binaries but cache_clean_cmd missing '{{binary}}' placeholder",
            config.name
        ));
    }

    // upgrade_cmd should contain {binary} if backend has multiple binaries
    if let Some(ref upgrade_cmd) = config.upgrade_cmd
        && needs_binary_placeholder
        && !upgrade_cmd.contains("{binary}")
    {
        ui::warning(&format!(
            "Backend '{}' has multiple binaries but upgrade_cmd missing '{{binary}}' placeholder",
            config.name
        ));
    }

    // Validate format-specific requirements
    match config.list_format {
        OutputFormat::Json | OutputFormat::JsonLines | OutputFormat::NpmJson => {
            if config.list_name_key.is_none() {
                return Err(DeclarchError::Other(
                    "JSON format requires 'name_key' to be specified in list block".to_string(),
                ));
            }
        }
        OutputFormat::JsonObjectKeys => {
            // JsonObjectKeys uses object keys as package names
            // Only requires version_key, not name_key
            if config.list_version_key.is_none() {
                return Err(DeclarchError::Other(
                    "JsonObjectKeys format requires 'version_key' to be specified in list block"
                        .to_string(),
                ));
            }
        }
        OutputFormat::Regex => {
            if config.list_regex.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'regex' to be specified in list block".to_string(),
                ));
            }
            if config.list_regex_name_group.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'name_group' to be specified".to_string(),
                ));
            }
        }
        OutputFormat::SplitWhitespace | OutputFormat::TabSeparated => {
            if config.list_name_col.is_none() {
                return Err(DeclarchError::Other(
                    "Whitespace/TSV format requires 'name_col' to be specified".to_string(),
                ));
            }
        }
        OutputFormat::Custom => {
            // Custom format doesn't require validation (uses Rust implementation)
        }
    }

    Ok(())
}
