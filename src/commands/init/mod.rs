//! Initialization commands
//!
//! This module handles all initialization-related commands:
//! - `declarch init` - Initialize root configuration
//! - `declarch init --backend <name>` - Add a backend
//! - `declarch init <path>` - Add a module
//! - `declarch init --list backends` - List available backends
//! - `declarch init --list modules` - List available modules
//! - `declarch init --restore-declarch` - Restore declarch.kdl

use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use std::fs;

/// Validate KDL content
pub fn validate_kdl(content: &str, context: &str) -> Result<()> {
    kdl::KdlDocument::parse(content)
        .map_err(|e| DeclarchError::ConfigError(format!("Invalid KDL in {}: {}", context, e)))?;
    Ok(())
}

pub mod backend;
pub mod list;
pub mod module;
pub mod root;
pub mod template;

/// Options for the init command
#[derive(Debug)]
pub struct InitOptions {
    /// Optional path for module initialization
    pub path: Option<String>,
    /// Hostname for root initialization
    pub host: Option<String>,
    /// Backend names to initialize
    pub backends: Vec<String>,
    /// Force overwrite existing files
    pub force: bool,
    /// Auto-confirm prompts
    pub yes: bool,
    /// Create local module (skip registry)
    pub local: bool,
}

fn normalize_backend_args(backends: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for item in backends {
        for part in item.split(',') {
            let name = part.trim();
            if !name.is_empty() {
                normalized.push(name.to_string());
            }
        }
    }
    normalized
}

/// Main entry point for init command
pub fn run(options: InitOptions) -> Result<()> {
    let normalized_backends = normalize_backend_args(&options.backends);

    // CASE A: MODULE INITIALIZATION
    if let Some(target_path) = options.path {
        return module::init_module(&target_path, options.force, options.yes, options.local);
    }

    // CASE A2: BACKEND INITIALIZATION (supports multiple)
    if !normalized_backends.is_empty() {
        let force = options.force || options.yes;
        let total = normalized_backends.len();

        // Ensure root config exists first
        let config_file = crate::utils::paths::config_file()?;
        if !config_file.exists() {
            root::init_root(options.host.clone(), force)?;
        }

        if total > 1 {
            output::header(&format!("Initializing {} backends", total));
        }

        for (i, backend_name) in normalized_backends.iter().enumerate() {
            if total > 1 {
                println!();
                output::info(&format!(
                    "[{}/{}] Initializing '{}'",
                    i + 1,
                    total,
                    backend_name
                ));
            }
            backend::init_backend(backend_name, force)?;
        }
        return Ok(());
    }

    // CASE B: ROOT INITIALIZATION
    root::init_root(options.host, options.force)
}

/// Restore declarch.kdl from template
///
/// Recreates declarch.kdl without affecting modules/ or backends/
pub fn restore_declarch(host: Option<String>) -> Result<()> {
    let config_file = crate::utils::paths::config_file()?;

    // Check if declarch is initialized
    let config_dir = crate::utils::paths::config_dir()?;
    if !config_dir.exists() {
        return Err(DeclarchError::Other(format!(
            "{} not initialized. Run '{}' first.",
            project_identity::DISPLAY_NAME,
            project_identity::cli_with("init")
        )));
    }

    output::header("Restoring Main Configuration");

    // Preserve existing imports if possible
    let existing_imports = if config_file.exists() {
        fs::read_to_string(&config_file).ok()
    } else {
        None
    };

    // Get hostname
    let hostname = host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    // Create from template
    let template = crate::utils::templates::default_host(&hostname);
    fs::write(&config_file, template)?;

    output::success(&format!("Restored: {}", config_file.display()));

    if existing_imports.is_some() {
        output::warning("Previous configuration overwritten. Check your module imports.");
    }

    Ok(())
}

// Re-export list functions for CLI
pub use list::is_module_available;
pub use list::list_available_backends;
pub use list::list_available_modules;

#[cfg(test)]
mod tests {
    use super::normalize_backend_args;

    #[test]
    fn normalize_backend_args_supports_comma_and_space_forms() {
        let input = vec![
            "pnpm,yarn".to_string(),
            "bun".to_string(),
            "  ".to_string(),
            "paru, yay".to_string(),
        ];
        let normalized = normalize_backend_args(&input);
        assert_eq!(
            normalized,
            vec![
                "pnpm".to_string(),
                "yarn".to_string(),
                "bun".to_string(),
                "paru".to_string(),
                "yay".to_string()
            ]
        );
    }
}
