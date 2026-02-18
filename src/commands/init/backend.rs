//! Backend initialization logic
//!
//! Handles the `declarch init --backend <name>` command flow:
//! 1. Fetch backend definition from remote registry
//! 2. Validate and sanitize backend name
//! 3. Write backend file to `backends/<name>.kdl`
//! 4. Add import to `declarch.kdl` in `backends { ... }`

use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use crate::utils::{paths, remote};
use presentation::print_backend_meta;
use std::fs;

mod meta_parser;
mod presentation;
mod root_import;
pub use root_import::add_backend_to_declarch;

/// Initialize a new backend configuration file
pub fn init_backend(backend_name: &str, force: bool) -> Result<()> {
    let root_dir = paths::config_dir()?;
    if output::is_verbose() {
        output::verbose(&format!("Config root: {}", root_dir.display()));
    }

    // Ensure declarch environment exists (auto-init if needed)
    super::root::ensure_environment()?;

    // Sanitize backend name
    let sanitized_name: String = backend_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if sanitized_name.is_empty() {
        return Err(DeclarchError::Other(
            "Invalid backend name. Use alphanumeric characters, hyphens, or underscores."
                .to_string(),
        ));
    }

    // Create backends directory
    let backends_dir = root_dir.join("backends");
    if !backends_dir.exists() {
        fs::create_dir_all(&backends_dir)?;
        if output::is_verbose() {
            output::verbose(&format!(
                "Created backend directory: {}",
                backends_dir.display()
            ));
        }
    }

    println!(
        "fetching '{}' from {}",
        sanitized_name,
        project_identity::REGISTRY_SLUG
    );

    let backend_content = match remote::fetch_backend_content(&sanitized_name) {
        Ok(content) => content,
        Err(e) => {
            if output::is_verbose() {
                output::verbose(&format!("Backend fetch error detail: {}", e));
            }
            return Err(DeclarchError::Other(format!(
                "failed to fetch backend '{}'. Please verify name/network and retry.\nHint: {}\nDetail: {}",
                sanitized_name,
                project_identity::cli_with("init --list backends"),
                e
            )));
        }
    };

    // Validate KDL (warning only, can bypass with --force)
    if let Err(e) = super::validate_kdl(&backend_content, &format!("backend '{}'", sanitized_name))
        && !force
    {
        output::warning(&format!("{}", e));
        output::info(&format!(
            "The backend may be malformed or incompatible with your {} version.",
            project_identity::BINARY_NAME
        ));
        output::info("You can still adopt it with --force, then edit the file manually.");

        if !output::prompt_yes_no("Continue with potentially invalid backend") {
            output::info("Cancelled. You can try a different backend or use --force to override.");
            return Ok(());
        }
    }

    // Parse and display meta information
    if let Ok(meta) = extract_backend_meta(&backend_content) {
        print_backend_meta(&meta);
    }

    // Check if file already exists
    let backend_file = backends_dir.join(format!("{}.kdl", sanitized_name));
    if backend_file.exists() && !force {
        output::warning(&format!(
            "Backend file already exists: {}",
            backend_file.display()
        ));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    if !force
        && !output::prompt_yes_no(&format!(
            "Are you sure you want this '{}' being adopted",
            sanitized_name
        ))
    {
        output::info("Cancelled.");
        return Ok(());
    }

    fs::write(&backend_file, &backend_content)?;
    if output::is_verbose() {
        output::verbose(&format!("Backend file written: {}", backend_file.display()));
    }

    // Always import directly to root config file.
    let root_config_path = root_dir.join(project_identity::CONFIG_FILE_BASENAME);
    match add_backend_to_declarch(&root_config_path, &sanitized_name) {
        Ok(true) => {
            println!("Backend '{}' adopted.", sanitized_name);
            if output::is_verbose() {
                output::verbose(&format!("Imported into: {}", root_config_path.display()));
            }
        }
        Ok(false) => {
            println!(
                "Backend '{}' fetched. Add to {} to use:",
                sanitized_name,
                project_identity::CONFIG_FILE_BASENAME
            );
            output::info(&format!("backends {{\"backends/{}.kdl\"}}", sanitized_name));
            if output::is_verbose() {
                output::verbose("No backends block found for automatic import.");
            }
        }
        Err(e) => {
            output::warning(&format!("Could not auto-import: {}", e));
            output::info(&format!(
                "Add manually to {}: backends {{\"backends/{}.kdl\"}}",
                project_identity::CONFIG_FILE_BASENAME,
                sanitized_name,
            ));
        }
    }

    Ok(())
}

/// Backend meta information extracted from KDL
#[derive(Debug, Default)]
pub struct BackendMeta {
    pub title: String,
    pub description: String,
    pub author: Option<String>,
    pub maintainers: Vec<String>,
    pub homepage: String,
    pub installation_guide: Option<String>,
    pub platforms: Vec<String>,
    pub requires: Vec<String>,
}

/// Extract meta information from backend KDL content
pub fn extract_backend_meta(content: &str) -> Result<BackendMeta> {
    meta_parser::extract_backend_meta_impl(content)
}

#[cfg(test)]
mod tests;
