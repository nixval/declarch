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

/// Load all backend configurations from backends.kdl
///
/// This function loads backends ONLY from backends.kdl using explicit imports.
/// There is NO auto-loading from backends/ directory - all files must be imported.
///
/// # Returns
/// - `Ok(HashMap)` - Map of backend name to configuration
/// - `Err` - If there's an error reading/parsing backend files
///
/// # Architecture
/// backends.kdl serves as the central aggregator with explicit imports:
/// ```kdl
/// import "backends/aur.kdl"
/// import "backends/flatpak.kdl"
/// // import "backends/npm.kdl"  // Comment out to disable
/// ```
///
/// # Duplicate Detection
/// If a backend is defined multiple times, a warning is shown and the last
/// loaded definition wins.
pub fn load_all_backends() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let mut backends: HashMap<String, BackendConfig> = HashMap::new();
    let mut backend_sources: Vec<BackendSource> = Vec::new();

    // Load backends ONLY from backends.kdl (with explicit imports)
    let backends_path = paths::backend_config()?;
    if backends_path.exists() {
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

    // Note: NO auto-loading from backends/ directory
    // All backends must be explicitly imported in backends.kdl

    Ok(backends)
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
