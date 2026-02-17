//! Install Command
//!
//! Adds packages to KDL configuration files and auto-syncs the system.

use crate::config::editor::{self, ConfigEditor};
use crate::config::loader;
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::packages::get_registry;
use crate::project_identity;
use crate::ui as output;
use crate::utils::paths;
use file_ops::{cleanup_install_backups, inject_import_to_root, rollback_install_edits};
use planning::plan_installs;

mod file_ops;
mod planning;

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
        output::header("Dry Run: Install");
        output::keyval("Packages", &planned_installs.len().to_string());
        output::keyval(
            "Target module",
            options
                .module
                .as_deref()
                .unwrap_or("modules/others.kdl (default)"),
        );
        output::keyval(
            "Auto sync",
            if options.no_sync {
                "disabled"
            } else {
                "enabled"
            },
        );

        for planned in &planned_installs {
            output::indent(&format!("+ {}:{}", planned.backend, planned.package), 1);
        }

        if options.verbose {
            output::separator();
            output::info("Resolution details:");
            for raw in &options.packages {
                output::indent(&format!("input: {}", raw), 1);
            }
        }

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
    let mut root_backup: Option<std::path::PathBuf> = None;
    let mut skipped_count = 0;

    // Step 3: Add each package to config
    let mut modified_modules: Vec<String> = Vec::new();

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

    // Step 3: Show summary of edits
    let mut files_created = Vec::new();
    let mut files_updated = Vec::new();
    let mut all_packages = Vec::new();

    for edit in &all_edits {
        // Track modified module for selective sync
        if let Some(module_name) = edit.file_path.file_stem()
            && let Some(module_str) = module_name.to_str()
            && !modified_modules.iter().any(|m| m == module_str)
        {
            modified_modules.push(module_str.to_string());
        }

        if edit.created_new_file {
            files_created.push(edit.file_path.display().to_string());

            // Auto-import new module to declarch.kdl
            let module_path = edit
                .file_path
                .strip_prefix(&paths::config_dir()?)
                .unwrap_or(&edit.file_path);
            if root_backup.is_none() {
                let config_path = paths::config_file()?;
                if config_path.exists() {
                    root_backup = Some(editor::backup_kdl_file(&config_path)?);
                }
            }
            inject_import_to_root(module_path)?;
        } else {
            files_updated.push(edit.file_path.display().to_string());
        }

        all_packages.extend(edit.packages_added.iter().cloned());
    }

    // Step 4: Auto-sync (unless --no-sync)
    if !options.no_sync {
        // Show sync message with package details
        let _packages_with_backend: Vec<String> =
            all_packages.iter().map(|p| p.to_string()).collect();

        // Import sync command at top to avoid circular dependency
        use crate::commands::sync::{self, SyncOptions};

        let sync_result = sync::run(SyncOptions {
            update: false,
            prune: false,
            dry_run: false,
            verbose: options.verbose,
            target: None,
            yes: options.yes,
            force: false,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: modified_modules.clone(),
            diff: false,
            format: None,
            output_version: None,
        });
        if options.verbose {
            output::verbose(&format!(
                "Auto-sync modules: {}",
                if modified_modules.is_empty() {
                    "(none)".to_string()
                } else {
                    modified_modules.join(", ")
                }
            ));
        }

        match sync_result {
            Ok(()) => {
                // Keep install transactional even if interruption happens late.
                if output::is_interrupted() {
                    rollback_install_edits(&all_edits, root_backup.as_ref());
                    output::info("Changes rolled back");
                    return Ok(());
                }
            }
            Err(crate::error::DeclarchError::Interrupted) => {
                // User cancelled - rollback and show friendly message
                rollback_install_edits(&all_edits, root_backup.as_ref());
                output::info("Changes rolled back");
                return Ok(());
            }
            Err(e) => {
                // Rollback all edits silently
                rollback_install_edits(&all_edits, root_backup.as_ref());
                return Err(e);
            }
        }
    }

    cleanup_install_backups(&all_edits, root_backup.as_ref());
    Ok(())
}

#[cfg(test)]
mod tests;
