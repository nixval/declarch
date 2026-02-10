//! Module initialization logic
//!
//! Handles the `declarch init <path>` command flow:
//! 1. Resolve module path
//! 2. Fetch from remote or use local template
//! 3. Write module file to `modules/<path>.kdl`
//! 4. Auto-inject import into root config

use crate::config::kdl::parse_kdl_content;
use crate::constants::CONFIG_EXTENSION;
use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use crate::utils::{self, paths, remote};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Initialize a new module
///
/// Flow:
/// 1. Auto-initialize root config if needed
/// 2. Resolve and validate module path
/// 3. Select template (hardcoded, remote, or local)
/// 4. Write module file
/// 5. Inject import into root config
pub fn init_module(target_path: &str, force: bool, yes: bool, local: bool) -> Result<()> {
    output::header("Importing Module");

    let root_dir = paths::config_dir()?;

    // Auto-initialize root if not exists
    if !root_dir.exists() {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        // Create root directory
        fs::create_dir_all(&root_dir)?;

        // Create default config
        let config_file = paths::config_file()?;
        let template = utils::templates::default_host(&hostname);
        fs::write(&config_file, template)?;
        output::success(&format!("Created config file: {}", config_file.display()));

        // Initialize state
        let _state = state::io::init_state(hostname.clone())?;
    }

    // 1. Resolve Path - preserve directory structure
    let mut path_buf = std::path::PathBuf::from(target_path);
    if path_buf.extension().is_none() {
        path_buf.set_extension(CONFIG_EXTENSION);
    }

    // Always prepend "modules/" to keep structure
    let modules_path = std::path::PathBuf::from("modules").join(&path_buf);
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

    // Check if user requested a namespaced module (e.g., "category/name")
    let is_registry_path = target_path.contains('/') || target_path.contains('\\');

    // STRATEGY A: Hardcoded Template (Fastest, Offline)
    let content = if let Some(local_tmpl) = utils::templates::get_template_by_name(&slug) {
        local_tmpl
    }
    // STRATEGY B: Remote Registry (The "Marketplace") - skip if --local flag
    else if is_registry_path && !local {
        // User explicitly requested a registry module - fail if not found
        match remote::fetch_module_content(target_path) {
            Ok(remote_content) => remote_content,
            Err(e) => {
                return Err(DeclarchError::ConfigError(format!(
                    "Failed to fetch module '{}' from registry: {}

\
                    Try one of these alternatives:
\
                    1. List available modules:    declarch init --list modules
\
                    2. Create local module:       declarch init --local {}
\
                    3. Use simple name:           declarch init {}",
                    target_path, e, slug, slug
                )));
            }
        }
    }
    // STRATEGY C: Local module creation (--local flag or simple name not in registry)
    else if local {
        // User explicitly wants local module with --local flag
        output::info(&format!("Creating local module: {}", slug));
        utils::templates::default_module(&slug)
    }
    // STRATEGY D: Check if simple name exists in registry
    else {
        // Simple name without namespace - check if it exists in registry
        let is_available = super::is_module_available(target_path);
        
        if !is_available {
            return Err(DeclarchError::ConfigError(format!(
                "Module '{}' not found in registry.

\
                Try one of these alternatives:
\
                1. List available modules:    declarch init --list modules
\
                2. Create local module:       declarch init --local {}
\
                3. Check module path:           declarch init <category>/{}",
                target_path, slug, slug
            )));
        }
        
        // Try to fetch from registry
        match remote::fetch_module_content(target_path) {
            Ok(remote_content) => remote_content,
            Err(e) => {
                return Err(DeclarchError::ConfigError(format!(
                    "Failed to fetch module '{}' from registry: {}",
                    target_path, e
                )));
            }
        }
    };

    // Display module meta information before proceeding
    display_module_meta(&content);

    // 4. Write File
    fs::write(&full_path, &content)?;

    // 5. AUTO INJECT IMPORT
    let root_config_path = paths::config_file()?;
    let import_path = modules_path.to_string_lossy().replace("\\", "/");
    inject_import_to_root(&root_config_path, &import_path, force, yes)?;

    output::success("Done");

    Ok(())
}

/// Extract and display meta information from KDL content
fn display_module_meta(content: &str) {
    // Try to parse the content and extract meta
    if let Ok(raw_config) = parse_kdl_content(content) {
        let meta = &raw_config.project_metadata;

        // Only display if we have some meta information
        let has_meta = meta.title.is_some()
            || meta.description.is_some()
            || meta.author.is_some()
            || meta.version.is_some()
            || !meta.tags.is_empty()
            || meta.url.is_some();

        if has_meta {
            output::separator();
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

            if !details.is_empty() {
                for detail in details {
                    println!("  {}", detail);
                }
            }

            output::separator();
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

    // Pattern to insert: "path/to/module.{extension}"
    // We add quotes for safety.
    let import_line = format!("    {:?}", import_path);

    // 1. Check if it already exists (Simple check)
    // This catches both active imports and commented ones.
    // We assume if the string is there, the user knows about it.
    if content.contains(import_path) {
        output::info(&format!(
            "Module '{}' is already referenced in config.",
            import_path
        ));
        return Ok(());
    }

    // Prompt for consent unless force or yes is active
    if !force && !yes
        && !output::prompt_yes_no(&format!(
            "Add '{}' to imports in {}?",
            import_path,
            crate::constants::CONFIG_FILE_NAME
        ))
    {
        output::info("Skipping auto-import. You can add it manually.");
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
        })
        .to_string()
    } else {
        // FALLBACK: Append new block if not found
        format!(
            "{}\n\nimports {{\n{}\n}}\n",
            content.trim_end(),
            import_line
        )
    };

    fs::write(config_path, new_content)?;

    Ok(())
}
