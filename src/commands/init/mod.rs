//! Initialization commands
//!
//! This module handles all initialization-related commands:
//! - `declarch init` - Initialize root configuration
//! - `declarch init --backend <name>` - Add a backend
//! - `declarch init <path>` - Add a module
//! - `declarch init --list backends` - List available backends
//! - `declarch init --list modules` - List available modules
//! - `declarch init --restore-backends` - Restore backends.kdl
//! - `declarch init --restore-declarch` - Restore declarch.kdl

use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
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

/// Main entry point for init command
pub fn run(options: InitOptions) -> Result<()> {
    // CASE A: MODULE INITIALIZATION
    if let Some(target_path) = options.path {
        return module::init_module(&target_path, options.force, options.yes, options.local);
    }

    // CASE A2: BACKEND INITIALIZATION (supports multiple)
    if !options.backends.is_empty() {
        let force = options.force || options.yes;
        let total = options.backends.len();
        
        // Ensure root config exists first
        let config_file = crate::utils::paths::config_file()?;
        if !config_file.exists() {
            root::init_root(options.host.clone(), force)?;
        }
        
        if total > 1 {
            output::header(&format!("Initializing {} backends", total));
        }
        
        for (i, backend_name) in options.backends.iter().enumerate() {
            if total > 1 {
                println!();
                output::info(&format!("[{}/{}] Initializing '{}'", i + 1, total, backend_name));
            }
            backend::init_backend(backend_name, force)?;
        }
        return Ok(());
    }

    // CASE B: ROOT INITIALIZATION
    root::init_root(options.host, options.force)
}

/// Restore backends.kdl from template
/// 
/// Recreates backends.kdl without affecting other config files
pub fn restore_backends() -> Result<()> {
    let backends_file = paths::backend_config()?;
    
    // Check if declarch is initialized
    let config_dir = paths::config_dir()?;
    if !config_dir.exists() {
        return Err(DeclarchError::Other(
            "Declarch not initialized. Run 'declarch init' first.".into(),
        ));
    }
    
    output::header("Restoring Backends Configuration");
    
    // Create from template
    let backends_template = backend::default_backends_kdl();
    fs::write(&backends_file, backends_template)?;
    
    output::success(&format!("Restored: {}", backends_file.display()));
    output::info("Your custom backends in backends/ folder are preserved.");
    output::warning("Official backends (aur, pacman, flatpak, npm) have been reset to defaults.");
    
    Ok(())
}

/// Restore declarch.kdl from template
///
/// Recreates declarch.kdl without affecting backends.kdl or modules/
pub fn restore_declarch(host: Option<String>) -> Result<()> {
    let config_file = paths::config_file()?;
    
    // Check if declarch is initialized
    let config_dir = paths::config_dir()?;
    if !config_dir.exists() {
        return Err(DeclarchError::Other(
            "Declarch not initialized. Run 'declarch init' first.".into(),
        ));
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
