//! Install Command
//!
//! Adds packages to KDL configuration files and auto-syncs the system.

use crate::config::editor::{self, ConfigEditor, restore_from_backup};
use crate::config::loader;
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use regex::Regex;
use std::fs;
use std::str::FromStr;

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
}



/// Run the install command
pub fn run(options: InstallOptions) -> Result<()> {
    if options.dry_run {
        output::header("Dry Run: Installing Packages");
        output::info("Would install the following packages:");
        for pkg in &options.packages {
            let module = options.module.as_deref().unwrap_or("others");
            output::indent(&format!("• {} → module: {}", pkg, module), 2);
        }
        if !options.no_sync {
            output::info("Would sync after installation");
        } else {
            output::info("Would NOT sync (no-sync flag set)");
        }
        return Ok(());
    }

    output::header("Installing Packages");

    // Step 1: Load existing config to check for duplicates
    let config_path = paths::config_file()?;
    let existing_packages = if config_path.exists() {
        let config = loader::load_root_config(&config_path)?;
        Some(config.packages)
    } else {
        None
    };

    // Step 2: Initialize config editor
    let editor = ConfigEditor::new();
    let mut all_edits = Vec::new();
    let mut skipped_count = 0;

    // Step 3: Add each package to config
    let mut modified_modules: Vec<String> = Vec::new();

    for package in &options.packages {
        // Parse package string (e.g., "soar:bat" → (Some("soar"), "bat"))
        let (backend_override, pkg_name) = editor::parse_package_string(package)?;

        // Determine backend: use override from package string, or --backend flag
        let backend_str = backend_override
            .as_ref()
            .or(options.backend.as_ref())
            .map(|s| s.as_str());

        // Step 3a: Check for exact match (same backend + name)
        let exact_match = if let Some(backend) = backend_str {
            if let Some(ref packages) = existing_packages {
                let pkg_id = PackageId {
                    backend: Backend::from_str(backend).unwrap_or_else(|_| Backend::Custom(backend.to_string())),
                    name: pkg_name.clone(),
                };
                packages.contains_key(&pkg_id)
            } else {
                false
            }
        } else {
            false
        };

        if exact_match {
            // Same package, same backend - skip immediately
            let target_backend = backend_str.unwrap_or("aur");

            output::warning(&format!(
                "Package '{}' (backend: {}) already exists in config, skipping",
                pkg_name, target_backend
            ));
            skipped_count += 1;
            continue;
        }

        // Step 3b: Check for cross-backend match (same name, different backend)
        let cross_backend_matches: Vec<_> = if let Some(ref packages) = existing_packages {
            packages
                .keys()
                .filter(|pkg_id| pkg_id.name == pkg_name)
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

            let target_backend = backend_str.unwrap_or("aur");

            output::warning(&format!(
                "Package '{}' already exists from: {}. Install from '{}' anyway?",
                pkg_name,
                existing_backends.join(", "),
                target_backend
            ));

            // Prompt user
            let should_continue = prompt_yes_no(
                &format!("Install {} from {}?", pkg_name, target_backend),
                false, // Default: no
            );

            if !should_continue {
                output::info("Skipped");
                skipped_count += 1;
                continue;
            }

            // User confirmed - proceed with installation
            output::info("Proceeding with installation...");
        }

        // Add package to config
        let edit = editor.add_package(&pkg_name, backend_str, options.module.as_deref())?;

        all_edits.push(edit);
    }

    // If all packages were skipped, no need to continue
    if skipped_count == options.packages.len() {
        output::info("All packages already exist in config");
        return Ok(());
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
            inject_import_to_root(module_path)?;
        } else {
            files_updated.push(edit.file_path.display().to_string());
        }

        all_packages.extend(edit.packages_added.iter().cloned());
    }

    // Step 4: Auto-sync (unless --no-sync)
    if !options.no_sync {
        // Show sync message with package details
        let packages_with_backend: Vec<String> = all_packages
            .iter()
            .map(|p| format!("{} ({})", p, options.backend.as_deref().unwrap_or("aur")))
            .collect();

        output::info(&format!(
            "Syncing packages: {} ...",
            packages_with_backend.join(", ")
        ));

        // Import sync command at top to avoid circular dependency
        use crate::commands::sync::{self, SyncOptions};

        let sync_result = sync::run(SyncOptions {
            update: false,
            prune: false,
            gc: false,
            dry_run: false,
            target: None,
            yes: options.yes,
            force: false,
            noconfirm: false,
            hooks: true, // Always run hooks during install
            skip_soar_install: false,
            modules: modified_modules.clone(), // Sync only modified modules
        });

        match sync_result {
            Ok(()) => {
                // Show success message with module name
                if let Some(module) = modified_modules.first() {
                    output::success(&format!("Sync completed, added to '{}.kdl'", module));
                } else {
                    output::success("Sync completed");
                }

                // Clean up backups on successful install
                for edit in &all_edits {
                    let Some(ref backup) = edit.backup_path else {
                        continue;
                    };
                    if let Err(e) = std::fs::remove_file(backup) {
                        output::warning(&format!("Failed to cleanup backup file: {}", e));
                    }
                }
            }
            Err(e) => {
                // Rollback all edits silently
                for edit in &all_edits {
                    if let Some(ref backup) = edit.backup_path {
                        // File existed before: restore from backup
                        let _ = restore_from_backup(backup);
                    } else if edit.created_new_file {
                        // New file: delete it
                        let _ = std::fs::remove_file(&edit.file_path);
                    }
                }

                return Err(e);
            }
        }
    } else {
        output::info("Skipped sync (--no-sync)");
        output::info("Run 'declarch sync' to apply changes");
    }

    Ok(())
}

/// Prompt user for yes/no confirmation
fn prompt_yes_no(question: &str, default: bool) -> bool {
    use std::io::{self, Write};

    let prompt = if default {
        format!("{} [Y/n] ", question)
    } else {
        format!("{} [y/N] ", question)
    };

    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim().to_lowercase();

    match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    }
}

/// Helper to inject the import statement into main config file
/// Auto-imports newly created modules so they're picked up by sync
fn inject_import_to_root(module_path: &std::path::Path) -> Result<()> {
    let config_path = paths::config_file()?;

    // Normalize path to use forward slashes in KDL (cross-platform)
    let import_path = module_path
        .components()
        .map(|comp| comp.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    // Skip if config file doesn't exist
    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;

    // Check if it already exists
    if content.contains(&import_path) {
        return Ok(());
    }

    // Normalize path to use forward slashes and remove .kdl extension for import
    let import_line = format!("    {:?}", import_path);

    // Regex Magic - same as init.rs
    let re = Regex::new(r#"(?m)^(.*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        // INJECT: Insert right after the opening brace
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

    fs::write(&config_path, new_content)?;

    Ok(())
}
