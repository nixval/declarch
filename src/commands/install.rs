//! Install Command
//!
//! Adds packages to KDL configuration files and auto-syncs the system.

use crate::config::editor::ConfigEditor;
use crate::config::loader;
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::packages::get_registry;
use crate::project_identity;
use crate::ui as output;
use crate::utils::paths;
use file_ops::cleanup_install_backups;
use finalize::{finalize_edits, run_auto_sync};
use planning::plan_installs;
use presentation::{show_dry_run_install, show_install_summary};

mod file_ops;
mod finalize;
mod planning;
mod presentation;

/// Options for the install command
#[derive(Debug)]
pub struct InstallOptions {
    /// Package(s) to install (format: [backend:]package)
    pub packages: Vec<String>,
    /// Target package manager
    pub backend: Option<String>,
    /// Target module file (e.g., "base" or "linux/notes")
    pub module: Option<String>,
    /// Don't sync after editing config
    pub no_sync: bool,
    /// Skip confirmation prompts
    pub yes: bool,
    /// Preview changes without executing
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
}

/// Run the install command
pub fn run(options: InstallOptions) -> Result<()> {
    let planned_installs = plan_installs(&options.packages, options.backend.as_ref())?;

    if options.dry_run {
        show_dry_run_install(&options, &planned_installs);
        return Ok(());
    }

    // Step 1: Load existing config to check for duplicates
    let config_path = paths::config_file()?;
    if options.verbose {
        output::verbose(&format!("Root config: {}", config_path.display()));
    }
    let existing_packages = if config_path.exists() {
        let config = loader::load_root_config(&config_path)?;
        Some(config.packages)
    } else {
        None
    };
    if options.verbose {
        output::info(&format!(
            "Install planning resolved {} package(s)",
            planned_installs.len()
        ));
    }

    // Step 2: Initialize config editor
    let editor = ConfigEditor::new();
    let mut all_edits = Vec::new();
    let mut skipped_count = 0;

    // Step 3: Add each package to config
    for planned in &planned_installs {
        let pkg_name = &planned.package;
        let backend_str = planned.backend.as_str();

        // Step 3a: Check for exact match (same backend + name)
        let exact_match = if let Some(ref packages) = existing_packages {
            let pkg_id = PackageId {
                backend: Backend::from(backend_str),
                name: pkg_name.clone(),
            };
            packages.contains_key(&pkg_id)
        } else {
            false
        };

        if exact_match {
            // Same package, same backend - skip immediately
            output::warning(&format!(
                "Package '{}' (backend: {}) already exists in config, skipping",
                pkg_name, backend_str
            ));
            skipped_count += 1;
            continue;
        }

        // Step 3b: Check for cross-backend match (same name, different backend)
        let cross_backend_matches: Vec<_> = if let Some(ref packages) = existing_packages {
            packages
                .keys()
                .filter(|pkg_id| pkg_id.name == *pkg_name)
                .collect()
        } else {
            Vec::new()
        };

        if !cross_backend_matches.is_empty() {
            // Package exists in different backend(s)
            let existing_backends: Vec<String> = cross_backend_matches
                .iter()
                .map(|pkg_id| pkg_id.backend.to_string())
                .collect();

            output::warning(&format!(
                "Package '{}' already exists from: {}. Install from '{}' anyway?",
                pkg_name,
                existing_backends.join(", "),
                backend_str
            ));

            // Prompt user
            let should_continue = output::prompt_yes_no_default(
                &format!("Install {} from {}?", pkg_name, backend_str),
                false,
            );

            if !should_continue {
                output::info("Skipped");
                skipped_count += 1;
                continue;
            }

            // User confirmed - proceed with installation
            output::info("Proceeding with installation...");
        }

        // Validate backend exists before adding to config
        let registry = get_registry();
        let registry_guard = registry
            .lock()
            .map_err(|e| DeclarchError::LockError(format!("Backend registry: {}", e)))?;

        // Check if backend config exists
        if !registry_guard.has_backend(backend_str) {
            output::warning(&format!(
                "Backend '{}' not found. Run '{}'",
                backend_str,
                project_identity::cli_with(&format!("init --backend {}", backend_str))
            ));
            skipped_count += 1;
            continue;
        }

        if let Some(backend_cfg) = registry_guard.get_backend_config(backend_str)
            && !crate::utils::platform::backend_supports_current_os(backend_cfg)
        {
            output::warning(&format!(
                "Skipping '{}:{}' because backend '{}' is not for this OS.",
                backend_str, pkg_name, backend_str
            ));
            output::info("Tip: keep it in config for your other machines.");
            skipped_count += 1;
            continue;
        }

        // Add package to config
        let edit = editor.add_package(pkg_name, Some(backend_str), options.module.as_deref())?;
        if options.verbose {
            output::verbose(&format!(
                "Planned edit: backend={} package={} file={}",
                backend_str,
                pkg_name,
                edit.file_path.display()
            ));
        }

        all_edits.push(edit);
    }

    // If all packages were skipped, no need to continue
    if skipped_count == options.packages.len() {
        output::info("All packages already exist in config");
        return Ok(());
    }
    if options.verbose {
        output::verbose(&format!("Skipped packages: {}", skipped_count));
        output::verbose(&format!("Applied edits: {}", all_edits.len()));
    }

    // Step 3: Finalize edits and inject imports for newly-created modules.
    let finalize = finalize_edits(&all_edits)?;

    // Step 4: Auto-sync (unless --no-sync)
    if !options.no_sync {
        run_auto_sync(
            &finalize.modified_modules,
            options.verbose,
            options.yes,
            &all_edits,
            finalize.root_backup.as_ref(),
        )?;
    }

    show_install_summary(options.verbose, &finalize.all_packages);
    cleanup_install_backups(&all_edits, finalize.root_backup.as_ref());
    Ok(())
}

#[cfg(test)]
mod tests;
