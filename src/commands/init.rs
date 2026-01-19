use crate::utils::{self, paths, remote, install};
use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use regex::Regex; // Import Regex

#[derive(Debug)]
pub struct InitOptions {
    pub path: Option<String>,
    pub host: Option<String>,
    pub force: bool,
    pub skip_soar_install: bool,
}

pub fn run(options: InitOptions) -> Result<()> {
    // CASE A: MODULE INITIALIZATION
    if let Some(target_path) = options.path {
        return init_module(&target_path, options.force);
    }

    // CASE B: ROOT INITIALIZATION (Keep existing logic)
    output::header("Initializing declarch root");

    // Check and install Soar if needed
    if !options.skip_soar_install && !install::is_soar_installed() {
        output::separator();
        output::warning("Soar is not installed");
        output::info("Soar is required for cross-distro package management");
        output::info("Installing Soar automatically...");

        if install::install_soar()? {
            output::separator();
        } else {
            output::separator();
            output::warning("Continuing without Soar - only AUR/Flatpak packages will work");
        }
    } else if install::is_soar_installed() {
        output::success("Soar is installed and ready");
    }

    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    if config_file.exists() && !options.force {
        output::warning("Configuration already exists.");
        output::info(&format!("Location: {}", config_file.display()));
        return Ok(());
    }

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        output::success(&format!("Created config directory: {}", config_dir.display()));
    }

    let hostname = options.host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let template = utils::templates::default_host(&hostname);

    fs::write(&config_file, template)?;
    output::success(&format!("Created config file: {}", config_file.display()));

    // Create modules/base.kdl with default template
    let modules_dir = config_dir.join("modules");
    if !modules_dir.exists() {
        fs::create_dir_all(&modules_dir)?;
        output::success(&format!("Created modules directory: {}", modules_dir.display()));
    }

    let base_module_path = modules_dir.join("base.kdl");
    if !base_module_path.exists() {
        let base_template = utils::templates::get_template_by_name("base")
            .unwrap_or_else(|| utils::templates::default_module("base"));
        fs::write(&base_module_path, base_template)?;
        output::success(&format!("Created base module: {}", base_module_path.display()));
    }

    let _state = state::io::init_state(hostname.clone())?;
    output::success(&format!("Initialized state for host: {}", hostname.green()));

    Ok(())
}

fn init_module(target_path: &str, force: bool) -> Result<()> {
    output::header("Importing Module");

    let root_dir = paths::config_dir()?;

    // Auto-initialize root if not exists
    if !root_dir.exists() {
        output::warning("Root config not found. Auto-initializing...");
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        // Create root directory
        fs::create_dir_all(&root_dir)?;
        output::success(&format!("Created config directory: {}", root_dir.display()));

        // Create default config
        let config_file = paths::config_file()?;
        let template = utils::templates::default_host(&hostname);
        fs::write(&config_file, template)?;
        output::success(&format!("Created config file: {}", config_file.display()));

        // Initialize state
        let _state = state::io::init_state(hostname.clone())?;
        output::success(&format!("Initialized state for host: {}", hostname.green()));
    }

    // 1. Resolve Path - preserve directory structure
    let mut path_buf = PathBuf::from(target_path);
    if path_buf.extension().is_none() {
        path_buf.set_extension("kdl");
    }

    // Always prepend "modules/" to keep structure
    let modules_path = PathBuf::from("modules").join(&path_buf);
    let full_path = root_dir.join(&modules_path);

    // 2. Check Existence
    if full_path.exists() && !force {
        output::warning(&format!("Module already exists: {}", full_path.display()));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    // Create parent directories
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 3. Select Template - use full target_path for remote fetch
    let slug = path_buf
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    output::info(&format!("Resolving module '{}'...", slug.cyan()));

    // STRATEGY A: Hardcoded Template (Fastest, Offline)
    let content = if let Some(local_tmpl) = utils::templates::get_template_by_name(&slug) {
        output::success("Using built-in template.");
        local_tmpl
    }
    // STRATEGY B: Remote Registry (The "Marketplace")
    else {
        output::info("Fetching from community registry...");
        // Use the full target_path (e.g., "hyprland/niri-nico") for remote fetch
        match remote::fetch_module_content(target_path) {
            Ok(remote_content) => {
                output::success("Module downloaded successfully.");
                remote_content
            },
            Err(e) => {
                output::warning(&format!("Remote fetch failed: {}", e));
                output::info("Falling back to generic empty module.");
                utils::templates::default_module(&slug)
            }
        }
    };

    // 4. Write File
    fs::write(&full_path, &content)?;
    output::success(&format!("Created module: {}", full_path.display()));

    // 5. AUTO INJECT IMPORT
    let root_config_path = paths::config_file()?;
    let import_path = modules_path.to_string_lossy().replace("\\", "/");
    inject_import_to_root(&root_config_path, &import_path)?;

    Ok(())
}

/// Helper to inject the import statement into declarch.kdl using Regex
fn inject_import_to_root(config_path: &Path, import_path: &str) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    
    // Pattern to insert: "path/to/module.kdl"
    // We add quotes for safety.
    let import_line = format!("    {:?}", import_path); 

    // 1. Check if it already exists (Simple check)
    // This catches both active imports and commented ones.
    // We assume if the string is there, the user knows about it.
    if content.contains(import_path) {
        output::info(&format!("Module '{}' is already referenced in config.", import_path));
        return Ok(());
    }

    // 2. Regex Magic
    // Find: imports { (with flexible whitespace)
    let re = Regex::new(r#"(?m)^(.*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        // INJECT: Insert right after the opening brace
        // $1 references the captured "imports {" line
        // We append a newline and our import line
        re.replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        }).to_string()
    } else {
        // FALLBACK: Append new block if not found
        format!("{}\n\nimports {{\n{}\n}}\n", content.trim_end(), import_line)
    };

    fs::write(config_path, new_content)?;
    output::success(&format!("Auto-imported: added '{}' to declarch.kdl", import_path.green()));

    Ok(())
}
