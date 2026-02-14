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
use crate::ui as output;
use crate::utils::paths;
use std::collections::HashMap;

/// Backend configuration type re-export
pub use crate::backends::config::BackendConfig;

/// Tracks where each backend was loaded from for duplicate detection
#[derive(Debug, Clone)]
struct BackendSource {
    name: String,
    source_file: String,
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
    let use_explicit_imports = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                // Quick check for 'backends "' or 'backends {' pattern
                content.contains("backends") && 
                    (content.contains("backends \"") || content.contains("backends {"))
            }
            Err(_) => false,
        }
    } else {
        false
    };

    if use_explicit_imports {
        // New way: Backends are loaded via config loader from declarch.kdl imports
        // This path should not be hit when called from commands that already loaded config
        // But we keep it for backward compatibility with direct calls
        output::info("Using explicit backend imports from declarch.kdl");
    }

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
                let existing = backend_sources.iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_backends_empty() {
        // In test environment, likely no backends configured
        let result = load_all_backends();
        assert!(result.is_ok());
        // Result may be empty or have backends depending on environment
    }
}
