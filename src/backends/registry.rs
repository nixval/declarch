//! # Backend Registry
//!
//! This module provides backend configuration loading.
//!
//! ## Architecture (v0.6+)
//!
//! **NO BUILT-IN BACKENDS**. All backends are loaded from user configuration:
//! - `~/.config/declarch/backends.kdl` - Main backend aggregator (import-based)
//!
//! Declarch is fully backend-agnostic. There are NO hardcoded backend definitions.
//! If no backends are configured, declarch will show an error directing the user
//! to install backends via `declarch init --backend <name>`.
//!
//! ## Explicit Import Architecture
//!
//! Unlike modules, backends use explicit import pattern:
//! - backends.kdl acts as the central aggregator
//! - All backend files must be explicitly imported: `import "backends/aur.kdl"`
//! - No auto-loading - user has full control over which backends are active
//! - Comment out imports to temporarily disable backends

use crate::backends::user_parser;
use crate::error::DeclarchError;
use crate::ui as output;
use crate::utils::paths;
use std::collections::HashMap;
use std::path::Path;

/// Backend configuration type re-export
pub use crate::backends::config::BackendConfig;

/// Tracks where each backend was loaded from for duplicate detection
#[derive(Debug, Clone)]
struct BackendSource {
    name: String,
    source_file: String,
}

fn has_explicit_backend_declaration(content: &str) -> bool {
    content.contains("backends")
        && (content.contains("backends \"") || content.contains("backends {"))
}

fn config_uses_explicit_backends(config_path: &Path) -> crate::error::Result<bool> {
    if !config_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(config_path)?;
    Ok(has_explicit_backend_declaration(&content))
}

fn strict_backend_mode_enabled() -> bool {
    std::env::var("DECLARCH_STRICT_BACKENDS")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Load all backend configurations
///
/// This function supports two modes:
/// 1. **New (Explicit Import)**: Backends are imported in declarch.kdl via `backends "backends.kdl"`
/// 2. **Legacy (Auto-load)**: Backends auto-loaded from `~/.config/declarch/backends.kdl`
///
/// Priority: If declarch.kdl exists and has backend imports, use those (new way).
/// Otherwise, fallback to auto-load from hardcoded path (legacy, deprecated).
///
/// # Returns
/// - `Ok(HashMap)` - Map of backend name to configuration
/// - `Err` - If there's an error reading/parsing backend files
///
/// # Migration
/// To migrate from legacy to explicit import, add to declarch.kdl:
/// ```kdl
/// backends "backends.kdl"
/// ```
pub fn load_all_backends() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let mut backends: HashMap<String, BackendConfig> = HashMap::new();
    let mut backend_sources: Vec<BackendSource> = Vec::new();

    // NEW: Check if declarch.kdl has explicit backend imports
    let config_path = paths::config_file()?;
    let use_explicit_imports = config_uses_explicit_backends(&config_path).unwrap_or(false);

    // Note: When using explicit imports, backends are loaded by config::loader
    // and should be accessed via MergedConfig.backends
    // This function primarily serves legacy auto-load mode now

    // LEGACY: Auto-load from backends.kdl (deprecated but supported for migration)
    let backends_path = paths::backend_config()?;
    if backends_path.exists() {
        // Check if this is legacy mode (no explicit imports in declarch.kdl)
        if !use_explicit_imports && config_path.exists() {
            output::warning("Auto-loading backends.kdl is deprecated.");
            output::info("Add the following to declarch.kdl to silence this warning:");
            output::info("  backends \"backends.kdl\"");
        }

        let user_backends = user_parser::load_user_backends(&backends_path)?;
        for config in user_backends {
            let source = BackendSource {
                name: config.name.clone(),
                source_file: backends_path.display().to_string(),
            };

            // Check for duplicates
            if backends.contains_key(&config.name) {
                let existing = backend_sources
                    .iter()
                    .find(|s| s.name == config.name)
                    .map(|s| s.source_file.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                output::warning(&format!(
                    "Duplicate backend '{}' defined in '{}' and '{}'. Using the later definition.",
                    config.name,
                    existing,
                    backends_path.display()
                ));
            }

            backend_sources.push(source);
            backends.insert(config.name.clone(), config);
        }
    }

    // Note: When using explicit imports, backends are loaded by config::loader
    // and should be accessed via MergedConfig.backends

    Ok(backends)
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
pub fn load_backends_from_config_with_sources()
-> crate::error::Result<(Vec<BackendConfig>, HashMap<String, Vec<String>>)> {
    use crate::utils::paths;

    let config_path = paths::config_file()?;

    if !config_path.exists() {
        return Ok((Vec::new(), HashMap::new()));
    }

    let config = crate::config::loader::load_root_config(&config_path)?;
    let backend_sources = config
        .backend_sources
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
        .collect();
    Ok((config.backends, backend_sources))
}

/// Load all backends using the best available method
///
/// Priority:
/// 1. If declarch.kdl has explicit backend imports (backends {} block), use those
/// 2. Otherwise, fall back to legacy load_all_backends() from backends.kdl
///
/// This provides seamless migration from legacy auto-load to import-based architecture.
pub fn load_all_backends_unified() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let config_path = paths::config_file()?;

    if config_uses_explicit_backends(&config_path)? {
        // Explicit import mode is authoritative: do not silently fall back.
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
        return Ok(map);
    }

    // Legacy method
    load_all_backends()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_explicit_backend_declaration() {
        assert!(has_explicit_backend_declaration(
            r#"backends "backends.kdl""#
        ));
        assert!(has_explicit_backend_declaration(
            r#"
            backends {
                "backends/aur.kdl"
            }
            "#
        ));
        assert!(!has_explicit_backend_declaration("pkg { paru { bat } }"));
    }

    #[test]
    fn test_load_all_backends_empty() {
        // In test environment, likely no backends configured
        let result = load_all_backends();
        assert!(result.is_ok());
        // Result may be empty or have backends depending on environment
    }
}
