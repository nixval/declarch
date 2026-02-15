//! Root initialization logic
//!
//! Handles the `declarch init` command (without arguments):
//! Creates the initial directory structure and config files atomically.

use crate::constants::CONFIG_EXTENSION;
use crate::error::Result;
use crate::state;
use crate::utils::{self, paths};
use std::fs;

/// Initialize root configuration directory
///
/// Creates atomically (all or nothing):
/// - ~/.config/declarch/ (config directory)
/// - ~/.config/declarch/backends/ (backend definitions)
/// - ~/.config/declarch/modules/ (module files)
/// - ~/.config/declarch/backends.kdl (default backends)
/// - ~/.config/declarch/declarch.kdl (main config)
/// - ~/.config/declarch/modules/base.kdl (default module)
pub fn init_root(host: Option<String>, force: bool) -> Result<()> {
    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    if config_file.exists() && !force {
        println!("Declarch is already initialized.");
        println!("  {}", config_dir.display());
        return Ok(());
    }

    let hostname = host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    // STEP 1: Prepare all content in memory (NO DISK OPERATIONS YET)
    let template = utils::templates::default_host(&hostname);
    let base_template = utils::templates::get_template_by_name("base")
        .unwrap_or_else(|| utils::templates::default_module("base"));
    let backends_kdl = super::backend::default_backends_kdl();

    // STEP 2: Initialize state first (may fail)
    let _state = state::io::init_state(hostname.clone())?;

    // STEP 3: Create directory structure
    let backends_dir = config_dir.join("backends");
    let modules_dir = config_dir.join("modules");

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&backends_dir)?;
    fs::create_dir_all(&modules_dir)?;

    // STEP 4: Write all files (atomic - all succeed or all fail)
    let backends_kdl_path = config_dir.join("backends.kdl");
    fs::write(&backends_kdl_path, backends_kdl)?;

    fs::write(&config_file, template)?;

    let base_module_path = modules_dir.join(format!("base.{}", CONFIG_EXTENSION));
    fs::write(&base_module_path, base_template)?;

    // Simple, clean output
    println!("Created declarch directory:");
    println!("  {}", config_dir.display());
    println!("Initializing declarch for host: {}", hostname);

    Ok(())
}

/// Ensure declarch environment exists, create if not
///
/// This is the unified function called by all init paths (module, backend, root)
/// to ensure the base environment is set up before proceeding.
///
/// Unlike `init_root`, this:
/// - Only creates if doesn't exist (never force overwrite)
/// - Shows minimal output (designed for behind-the-scenes use)
/// - Returns whether it created new environment or not
pub fn ensure_environment() -> Result<bool> {
    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    // Already initialized, nothing to do
    if config_file.exists() {
        return Ok(false);
    }

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown".to_string());

    // Prepare content
    let template = utils::templates::default_host(&hostname);
    let base_template = utils::templates::get_template_by_name("base")
        .unwrap_or_else(|| utils::templates::default_module("base"));
    let backends_kdl = super::backend::default_backends_kdl();

    // Initialize state
    let _state = state::io::init_state(hostname)?;

    // Create directories
    let backends_dir = config_dir.join("backends");
    let modules_dir = config_dir.join("modules");

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&backends_dir)?;
    fs::create_dir_all(&modules_dir)?;

    // Write files
    let backends_kdl_path = config_dir.join("backends.kdl");
    fs::write(&backends_kdl_path, backends_kdl)?;
    fs::write(&config_file, template)?;

    let base_module_path = modules_dir.join(format!("base.{}", CONFIG_EXTENSION));
    fs::write(&base_module_path, base_template)?;

    // Show minimal output for behind-the-scenes operation
    crate::ui::success(&format!("Created config file: {}", config_file.display()));
    crate::ui::success(&format!(
        "Created backends file: {}",
        backends_kdl_path.display()
    ));

    Ok(true)
}
