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
mod tests;
