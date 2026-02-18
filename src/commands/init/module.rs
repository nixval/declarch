//! Module initialization logic
//!
//! Handles the `declarch init <path>` command flow:
//! 1. Resolve module path
//! 2. Fetch from remote or use local template
//! 3. Validate KDL (warning, bypassable with --force)
//! 4. Write module file to `modules/<path>.kdl`
//! 5. Auto-inject import into root config

use crate::constants::CONFIG_EXTENSION;
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::utils::{self, paths, remote};
use presentation::{display_module_meta, inject_import_to_root};
use std::fs;

mod presentation;

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
    if let Err(e) = super::validate_kdl(&content, &format!("module '{}'", target_path))
        && !force
    {
        crate::ui::warning(&format!("{}", e));
        crate::ui::info(&format!(
            "The module may be malformed or incompatible with your {} version.",
            project_identity::BINARY_NAME
        ));
        crate::ui::info("You can still import it with --force, then edit the file manually.");

        if !crate::ui::prompt_yes_no("Continue with potentially invalid module") {
            crate::ui::info(
                "Cancelled. You can try a different module or use --force to override.",
            );
            return Ok(());
        }
    }

    // STEP 5: Display meta before proceeding
    display_module_meta(&content);

    // STEP 6: Ensure root environment exists (auto-init if needed)
    super::root::ensure_environment()?;

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
  1. List available modules:    {}
  2. Create local module:       {}
  3. Use simple name:           {}",
                target_path,
                e,
                project_identity::cli_with("init --list modules"),
                project_identity::cli_with(&format!("init --local {}", slug)),
                project_identity::cli_with(&format!("init {}", slug))
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
  1. List available modules:    {}
  2. Create local module:       {}
  3. Check module path:         {}",
            target_path,
            project_identity::cli_with("init --list modules"),
            project_identity::cli_with(&format!("init --local {}", slug)),
            project_identity::cli_with(&format!("init <category>/{}", slug))
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
