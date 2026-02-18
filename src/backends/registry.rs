//! # Backend Registry
//!
//! This module provides backend configuration loading.
//!
//! ## Architecture (v0.6+)
//!
//! **NO BUILT-IN BACKENDS**. All backends are loaded from user configuration:
//! - `~/.config/declarch/declarch.kdl` via `backends { ... }` imports.
//!
//! Declarch is fully backend-agnostic. There are NO hardcoded backend definitions.
//! If no backends are configured, declarch will show an error directing the user
//! to install backends via `declarch init --backend <name>`.

use crate::error::DeclarchError;
use crate::project_identity;
use crate::ui as output;
use crate::utils::paths;
use std::collections::HashMap;

/// Backend configuration type re-export
pub use crate::backends::config::BackendConfig;
type BackendSourceMap = HashMap<String, Vec<String>>;
type BackendsWithSources = (Vec<BackendConfig>, BackendSourceMap);

fn strict_backend_mode_enabled() -> bool {
    project_identity::env_get("STRICT_BACKENDS")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn load_all_backends() -> crate::error::Result<HashMap<String, BackendConfig>> {
    load_all_backends_unified()
}

/// Filesystem-based backend registry implementing the BackendRegistry trait
pub struct FilesystemBackendRegistry;

impl crate::traits::BackendRegistry for FilesystemBackendRegistry {
    fn load_all(&self) -> crate::error::Result<HashMap<String, BackendConfig>> {
        load_all_backends()
    }

    fn get(&self, name: &str) -> crate::error::Result<Option<BackendConfig>> {
        let all = self.load_all()?;
        Ok(all.get(name).cloned())
    }

    fn has(&self, name: &str) -> bool {
        self.get(name).map(|b| b.is_some()).unwrap_or(false)
    }
}

/// Load backends from declarch.kdl config (import-based architecture)
///
/// This is the new way to load backends when using explicit imports in declarch.kdl.
/// It loads the config file and returns the backends defined in the 'backends {}' block.
///
/// Returns empty vector if:
/// - Config file doesn't exist
/// - No backends are defined in the config
pub fn load_backends_from_config() -> crate::error::Result<Vec<BackendConfig>> {
    let (backends, _) = load_backends_from_config_with_sources()?;
    Ok(backends)
}

/// Load backends and source paths from declarch.kdl config (import-based architecture).
pub fn load_backends_from_config_with_sources() -> crate::error::Result<BackendsWithSources> {
    use crate::utils::paths;

    let config_path = paths::config_file()?;

    if !config_path.exists() {
        return Ok((Vec::new(), HashMap::new()));
    }

    let config = crate::config::loader::load_root_config(&config_path)?;
    let backend_sources = build_backend_source_map(config.backend_sources);
    Ok((config.backends, backend_sources))
}

fn build_backend_source_map(
    backend_sources: HashMap<String, Vec<std::path::PathBuf>>,
) -> BackendSourceMap {
    backend_sources
        .into_iter()
        .map(|(name, paths)| {
            (
                name,
                paths
                    .into_iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// Load all backends using the best available method
///
pub fn load_all_backends_unified() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Ok(HashMap::new());
    }

    let (config_backends, source_map) = load_backends_from_config_with_sources()?;
    let mut map = HashMap::new();
    let mut duplicate_names = Vec::new();
    for backend in config_backends {
        let name = backend.name.clone();
        if map.insert(name.clone(), backend).is_some() {
            duplicate_names.push(name);
        }
    }

    if !duplicate_names.is_empty() {
        duplicate_names.sort();
        duplicate_names.dedup();
        let detail_lines = duplicate_names
            .iter()
            .map(|name| {
                let sources = source_map.get(name).cloned().unwrap_or_default();
                let winner = sources
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "unknown source".to_string());
                if sources.is_empty() {
                    format!("  - {}: winner={}", name, winner)
                } else {
                    format!(
                        "  - {}: sources=[{}], winner={}",
                        name,
                        sources.join(", "),
                        winner
                    )
                }
            })
            .collect::<Vec<_>>();
        let details = detail_lines.join("\n");
        if strict_backend_mode_enabled() {
            return Err(DeclarchError::ConfigError(format!(
                "Duplicate backend definitions found in explicit imports:\n{}",
                details
            )));
        }
        output::warning(&format!(
            "Duplicate backend definitions found; using the last imported definition:\n{}",
            details
        ));
    }
    Ok(map)
}

#[cfg(test)]
mod tests;
