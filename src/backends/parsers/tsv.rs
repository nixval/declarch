use crate::backends::config::BackendConfig;
use crate::core::types::PackageMetadata;
use crate::error::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Parse tab-separated output (like flatpak list --columns=...)
pub fn parse_tsv(output: &str, config: &BackendConfig) -> Result<HashMap<String, PackageMetadata>> {
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

            installed.insert(
                name.to_string(),
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
mod tests;
