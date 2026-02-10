//! Initialization commands
//!
//! This module handles all initialization-related commands:
//! - `declarch init` - Initialize root configuration
//! - `declarch init --backend <name>` - Add a backend
//! - `declarch init <path>` - Add a module
//! - `declarch init --list backends` - List available backends
//! - `declarch init --list modules` - List available modules

use crate::error::Result;
use crate::ui as output;

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

// Re-export list functions for CLI
pub use list::is_module_available;
pub use list::list_available_backends;
pub use list::list_available_modules;
