//! Root initialization logic
//!
//! Handles the `declarch init` command (without arguments):
//! Creates the initial directory structure and config files.

use crate::constants::CONFIG_EXTENSION;
use crate::error::Result;
use crate::state;
use crate::utils::{self, paths};
use std::fs;

/// Initialize root configuration directory
///
/// Creates:
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

    // Create directory structure
    let backends_dir = config_dir.join("backends");
    let modules_dir = config_dir.join("modules");
    
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&backends_dir)?;
    fs::create_dir_all(&modules_dir)?;

    // Create default files
    let backends_kdl_path = config_dir.join("backends.kdl");
    if !backends_kdl_path.exists() {
        fs::write(&backends_kdl_path, super::backend::default_backends_kdl())?;
    }

    let template = utils::templates::default_host(&hostname);
    fs::write(&config_file, template)?;

    let base_module_path = modules_dir.join(format!("base.{}", CONFIG_EXTENSION));
    if !base_module_path.exists() {
        let base_template = utils::templates::get_template_by_name("base")
            .unwrap_or_else(|| utils::templates::default_module("base"));
        fs::write(&base_module_path, base_template)?;
    }

    let _state = state::io::init_state(hostname.clone())?;
    
    // Simple, clean output
    println!("Created declarch directory:");
    println!("  {}", config_dir.display());
    println!("Initializing declarch for host: {}", hostname);

    Ok(())
}
