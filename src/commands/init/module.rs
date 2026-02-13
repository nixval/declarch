//! Module initialization logic
//!
//! Handles the `declarch init <path>` command flow:
//! 1. Resolve module path
//! 2. Fetch from remote or use local template
//! 3. Validate KDL (warning, bypassable with --force)
//! 4. Write module file to `modules/<path>.kdl`
//! 5. Auto-inject import into root config

use crate::config::kdl::parse_kdl_content;
use crate::constants::CONFIG_EXTENSION;
use crate::error::{DeclarchError, Result};
use crate::utils::{self, paths, remote};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Initialize a new module
///
/// Flow:
/// 1. Resolve and validate module path
/// 2. Fetch/resolve template content (no disk operations yet)
/// 3. Ensure root config exists
/// 4. Write module file and inject import
pub fn init_module(target_path: &str, force: bool, yes: bool, local: bool) -> Result<()> {
    crate::ui::header("Importing Module");

    // STEP 1: Resolve module path
    let mut path_buf = std::path::PathBuf::from(target_path);
    if path_buf.extension().is_none() {
        path_buf.set_extension(CONFIG_EXTENSION);
    }

    // Prepend "modules/" only if not already present (avoid "modules/modules/")
    let modules_path = if path_buf.starts_with("modules") {
        path_buf.clone()
    } else {
        std::path::PathBuf::from("modules").join(&path_buf)
    };
    let root_dir = paths::config_dir()?;
    let full_path = root_dir.join(&modules_path);

    // STEP 2: Check if already exists
    if full_path.exists() && !force {
        crate::ui::warning(&format!("Module already exists: {}", full_path.display()));
        crate::ui::info("Use --force to overwrite.");
        return Ok(());
    }

    // STEP 3: Resolve template content (NO DISK OPERATIONS YET)
    let slug = path_buf
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let is_registry_path = target_path.contains('/') || target_path.contains('\\');

    let content = resolve_module_content(target_path, &slug, is_registry_path, local)?;

    // STEP 4: Validate KDL (warning only, can bypass with --force)
    if let Err(e) = super::validate_kdl(&content, &format!("module '{}'", target_path)) {
        if !force {
            crate::ui::warning(&format!("{}", e));
            crate::ui::info("The module may be malformed or incompatible with your declarch version.");
            crate::ui::info("You can still import it with --force, then edit the file manually.");
            
            if !crate::ui::prompt_yes_no("Continue with potentially invalid module") {
                crate::ui::info("Cancelled. You can try a different module or use --force to override.");
                return Ok(());
            }
        }
    }

    // STEP 5: Display meta before proceeding
    display_module_meta(&content);

    // STEP 6: Ensure root config exists (or create it)
    ensure_root_config()?;

    // STEP 7: Create directories and write file (KDL already validated or bypassed)
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&full_path, &content)?;

    // STEP 8: Inject import
    let root_config_path = paths::config_file()?;
    let import_path = modules_path.to_string_lossy().replace("\\", "/");
    inject_import_to_root(&root_config_path, &import_path, force, yes)?;

    crate::ui::success("Done");

    Ok(())
}

/// Resolve module content without any disk operations
fn resolve_module_content(
    target_path: &str,
    slug: &str,
    is_registry_path: bool,
    local: bool,
) -> Result<String> {
    // STRATEGY A: Hardcoded Template
    if let Some(local_tmpl) = utils::templates::get_template_by_name(slug) {
        return Ok(local_tmpl);
    }

    // STRATEGY B: Remote Registry
    if is_registry_path && !local {
        return match remote::fetch_module_content(target_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(DeclarchError::ConfigError(format!(
                "Failed to fetch module '{}' from registry: {}

Try one of these alternatives:
  1. List available modules:    declarch init --list modules
  2. Create local module:       declarch init --local {}
  3. Use simple name:           declarch init {}",
                target_path, e, slug, slug
            ))),
        };
    }

    // STRATEGY C: Local module creation
    if local {
        crate::ui::info(&format!("Creating local module: {}", slug));
        return Ok(utils::templates::default_module(slug));
    }

    // STRATEGY D: Check registry availability
    let is_available = super::is_module_available(target_path);
    
    if !is_available {
        return Err(DeclarchError::ConfigError(format!(
            "Module '{}' not found in registry.

Try one of these alternatives:
  1. List available modules:    declarch init --list modules
  2. Create local module:       declarch init --local {}
  3. Check module path:         declarch init <category>/{}",
            target_path, slug, slug
        )));
    }

    // Try to fetch from registry
    match remote::fetch_module_content(target_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(DeclarchError::ConfigError(format!(
            "Failed to fetch module '{}' from registry: {}",
            target_path, e
        ))),
    }
}

/// Ensure root config exists, create if not
fn ensure_root_config() -> Result<()> {
    let root_dir = paths::config_dir()?;
    
    if root_dir.exists() {
        return Ok(());
    }

    // Create fresh config
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown".to_string());

    fs::create_dir_all(&root_dir)?;

    let config_file = paths::config_file()?;
    let template = utils::templates::default_host(&hostname);
    fs::write(&config_file, template)?;
    
    crate::ui::success(&format!("Created config file: {}", config_file.display()));

    // Also create default backends.kdl
    let backends_file = paths::backend_config()?;
    let backends_template = crate::commands::init::backend::default_backends_kdl();
    fs::write(&backends_file, backends_template)?;
    crate::ui::success(&format!("Created backends file: {}", backends_file.display()));

    let _ = crate::state::io::init_state(hostname)?;

    Ok(())
}

/// Extract and display meta information from KDL content
fn display_module_meta(content: &str) {
    if let Ok(raw_config) = parse_kdl_content(content) {
        let meta = &raw_config.project_metadata;

        let has_meta = meta.title.is_some()
            || meta.description.is_some()
            || meta.author.is_some()
            || meta.version.is_some()
            || !meta.tags.is_empty()
            || meta.url.is_some();

        if has_meta {
            crate::ui::separator();
            println!("{}", "Module Information:".bold().cyan());

            if let Some(title) = &meta.title {
                println!("  {}", title.bold());
                println!();
            }

            if let Some(description) = &meta.description {
                println!("  {}", description.dimmed());
                println!();
            }

            let mut details = Vec::new();

            if let Some(author) = &meta.author {
                details.push(format!("Author: {}", author.yellow()));
            }

            if let Some(version) = &meta.version {
                details.push(format!("Version: {}", version.green()));
            }

            if !meta.tags.is_empty() {
                details.push(format!("Tags: {}", meta.tags.join(", ").purple()));
            }

            if let Some(url) = &meta.url {
                details.push(format!("URL: {}", url.blue().underline()));
            }

            for detail in details {
                println!("  {}", detail);
            }

            crate::ui::separator();
        }
    }
}

/// Helper to inject the import statement into main config file using Regex
fn inject_import_to_root(
    config_path: &Path,
    import_path: &str,
    force: bool,
    yes: bool,
) -> Result<()> {
    let content = fs::read_to_string(config_path)?;

    let import_line = format!("    {:?}", import_path);

    // Check if already exists (only active imports, not commented)
    let active_import_pattern = format!(r#"(?m)^\s+"{}"\s*$"#, regex::escape(import_path));
    if Regex::new(&active_import_pattern)
        .map(|re| re.is_match(&content))
        .unwrap_or(false)
    {
        crate::ui::info(&format!(
            "Module '{}' is already imported in config.",
            import_path
        ));
        return Ok(());
    }

    // Prompt for consent
    if !force && !yes
        && !crate::ui::prompt_yes_no(&format!(
            "Add '{}' to imports in {}?",
            import_path,
            crate::constants::CONFIG_FILE_NAME
        ))
    {
        crate::ui::info("Skipping auto-import. You can add it manually.");
        return Ok(());
    }

    // Inject import (non-greedy match for imports block)
    let re = Regex::new(r#"(?m)^(\s*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        re.replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        })
        .to_string()
    } else {
        format!(
            "{}\n\nimports {{\n{}\n}}\n",
            content.trim_end(),
            import_line
        )
    };

    fs::write(config_path, new_content)?;

    Ok(())
}
