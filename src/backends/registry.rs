//! # Backend Registry
//!
//! This module provides backend configuration loading.
//!
//! ## Architecture (v0.6+)
//!
//! **NO BUILT-IN BACKENDS**. All backends are loaded from user configuration:
//! - `~/.config/declarch/backends/*.kdl` - User backend definitions
//!
//! Declarch is fully backend-agnostic. There are NO hardcoded backend definitions.
//! If no backends are configured, declarch will show an error directing the user
//! to install backends via `declarch init --backend <name>`.

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

/// Load all backend configurations from user config directory
///
/// This function ONLY loads user-defined backends from `~/.config/declarch/backends/*.kdl`.
/// There are no built-in backends - declarch is fully agnostic.
///
/// # Returns
/// - `Ok(HashMap)` - Map of backend name to configuration
/// - `Err` - If there's an error reading/parsing backend files
///
/// # Note
/// If no backends are found, returns an empty HashMap. The caller should
/// check for empty result and show appropriate error message.
///
/// # Duplicate Detection
/// If a backend is defined multiple times (e.g., both in backends.kdl and
/// backends/aur.kdl), a warning is shown and the last loaded definition wins.
pub fn load_all_backends() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let mut backends: HashMap<String, BackendConfig> = HashMap::new();
    let mut backend_sources: Vec<BackendSource> = Vec::new();

    // Load user-defined backends from backends.kdl
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

    // Load individual backend files from backends/ directory
    let backends_dir = paths::config_dir()?.join("backends");
    if backends_dir.exists() {
        for entry in std::fs::read_dir(&backends_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only process .kdl files
            if path.extension().map(|e| e == "kdl").unwrap_or(false) {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                match load_backend_file(&path) {
                    Ok(Some(config)) => {
                        // Check for duplicates
                        if backends.contains_key(&config.name) {
                            let existing = backend_sources.iter()
                                .find(|s| s.name == config.name)
                                .map(|s| s.source_file.clone())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            output::warning(&format!(
                                "Duplicate backend '{}' defined in '{}' and '{}'. Using '{}'.",
                                config.name,
                                existing,
                                file_name,
                                file_name
                            ));
                        }
                        
                        backend_sources.push(BackendSource {
                            name: config.name.clone(),
                            source_file: file_name.clone(),
                        });
                        backends.insert(config.name.clone(), config);
                    }
                    Ok(None) => {
                        // File doesn't exist or empty
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load backend file '{}': {}", 
                            path.display(), e);
                        // Continue loading other backends
                    }
                }
            }
        }
    }

    Ok(backends)
}

/// Load a single backend configuration file
fn load_backend_file(path: &std::path::Path) -> crate::error::Result<Option<BackendConfig>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| crate::error::DeclarchError::Other(
            format!("Failed to read backend file {}: {}", path.display(), e)
        ))?;
    
    // Parse single backend from file
    // The backends/ directory contains individual backend files, not an aggregator
    user_parser::parse_backend_file(&content)
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
