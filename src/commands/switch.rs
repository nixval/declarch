use crate::config::types::GlobalConfig;
use crate::core::types::Backend;
use crate::error::{DeclarchError, Result};
use crate::packages::{PackageManager, create_manager};
use crate::state::{self, types::PackageState};
use crate::ui as output;
use chrono::Utc;
use colored::Colorize;
use std::process::Command;

#[derive(Debug)]
pub struct SwitchOptions {
    pub old_package: String,
    pub new_package: String,
    pub backend: Option<String>,
    pub dry_run: bool,
    pub yes: bool,
    pub force: bool,
}

pub fn run(options: SwitchOptions) -> Result<()> {
    output::header("Switching Package Variant");

    // ==========================================
    // PHASE 1: VALIDATION & DISCOVERY
    // ==========================================

    output::info("Analyzing package transition...");
    output::separator();

    // 1. Load current state
    let mut state = state::io::load_state()?;

    // 2. Determine backend
    let backend = determine_backend(&options.old_package, options.backend)?;

    // 3. Get package manager
    let global_config = GlobalConfig::default();

    let manager: Box<dyn PackageManager> = create_manager(&backend, &global_config, false)
        .map_err(|e| DeclarchError::Other(format!("Failed to create package manager: {}", e)))?;

    // Check manager availability
    if !manager.is_available() {
        return Err(DeclarchError::Other(format!(
            "Backend '{}' is not available on this system",
            backend
        )));
    }

    // 4. Get installed packages
    let installed = manager.list_installed()?;

    // 5. Validate old package exists
    let old_state_key = format!("{}:{}", backend, options.old_package);
    let old_state_entry = state.packages.get(&old_state_key);

    if old_state_entry.is_none() && !options.force {
        return Err(DeclarchError::Other(format!(
            "Package '{}' is not tracked by declarch. Use --force to override.",
            options.old_package
        )));
    }

    // Check if old package is actually installed
    let old_installed = installed.keys().any(|name| name == &options.old_package);
    if !old_installed {
        return Err(DeclarchError::Other(format!(
            "Package '{}' is not installed on the system",
            options.old_package
        )));
    }

    // 6. Check if new package is already installed
    let new_installed = installed.keys().any(|name| name == &options.new_package);
    if new_installed && !options.force {
        return Err(DeclarchError::Other(format!(
            "Package '{}' is already installed. Use --force to override.",
            options.new_package
        )));
    }

    // 7. Check reverse dependencies
    output::info("Checking for dependencies...");
    match manager.get_required_by(&options.old_package) {
        Ok(required_by) if !required_by.is_empty() => {
            output::warning(&format!(
                "The following package(s) depend on '{}': {}",
                options.old_package,
                required_by.join(", ")
            ));
            output::warning("Removing or replacing this package may break these dependencies.");
            output::indent("Ensure the new package provides the same functionality.", 1);

            if !options.yes
                && !options.force
                && !output::prompt_yes_no("Continue despite dependency warnings?")
            {
                output::warning("Transition cancelled by user");
                return Ok(());
            }
        }
        Ok(_) => {
            // No reverse dependencies found
            output::success("No reverse dependencies found");
        }
        Err(e) => {
            // Error checking dependencies, warn but continue
            output::warning(&format!("Could not check reverse dependencies: {}", e));
            output::warning("Proceed with caution");
        }
    }

    // ==========================================
    // PHASE 2: TRANSACTION PLANNING
    // ==========================================

    output::separator();
    output::info("Transition plan:");
    output::indent(
        &format!("{} {}", "Remove:".red().bold(), options.old_package),
        1,
    );
    output::indent(
        &format!("{} {}", "Install:".green().bold(), options.new_package),
        1,
    );
    output::indent(&format!("Backend: {}", backend), 1);
    output::separator();

    if !options.yes && !options.dry_run && !output::prompt_yes_no("Proceed with transition?") {
        output::warning("Transition cancelled by user");
        return Ok(());
    }

    if options.dry_run {
        output::info("Dry run completed - no changes made");
        return Ok(());
    }

    // ==========================================
    // PHASE 3: EXECUTION WITH ROLLBACK
    // ==========================================

    // Create in-memory backup before changing anything
    // Note: This provides rollback protection within this process only.
    // For concurrent process protection, file-level locking would be needed.
    // TODO: Implement proper transaction support with file locking for multi-process safety.
    let state_backup = state.clone();
    output::info("Created state backup");

    match execute_transition(
        &options.old_package,
        &options.new_package,
        &backend,
        &*manager,
    ) {
        Ok(()) => {
            // ==========================================
            // PHASE 4: UPDATE STATE
            // ==========================================

            output::info("Updating declarch state...");

            // Remove old package from state
            state.packages.remove(&old_state_key);

            // Add new package to state
            let new_state_key = format!("{}:{}", backend, options.new_package);

            // Discover actual AUR package name if applicable
            let aur_package_name = if backend == Backend::Aur {
                discover_aur_package_name(&options.new_package)
            } else {
                None
            };

            let new_pkg_state = PackageState {
                backend: backend.clone(),
                config_name: options.new_package.clone(),
                provides_name: options.new_package.clone(),
                aur_package_name,
                installed_at: Utc::now(),
                version: installed
                    .get(&options.new_package)
                    .and_then(|m| m.version.clone()),
            };

            state.packages.insert(new_state_key, new_pkg_state);

            // Update metadata
            state.meta.last_sync = Utc::now();

            // Save state with file locking
            state::io::save_state_locked(&state)?;

            output::separator();
            output::success(&format!(
                "Transition complete: {} → {}",
                options.old_package.cyan(),
                options.new_package.green()
            ));

            Ok(())
        }

        Err(e) => {
            // ==========================================
            // ROLLBACK ON FAILURE
            // ==========================================

            output::error(&format!("Transition failed: {}", e));
            output::warning("Rolling back state changes...");

            // Restore state from backup (with locking)
            if let Err(e2) = state::io::save_state_locked(&state_backup) {
                output::error(&format!("Failed to restore state: {}", e2));
                return Err(DeclarchError::Other(format!(
                    "Transition failed and state rollback failed: {} - {}",
                    e, e2
                )));
            }

            // Attempt to restore old package if it was removed
            if installed.keys().all(|name| name != &options.old_package) {
                output::warning(&format!("Attempting to restore {}...", options.old_package));
                if let Err(e2) = manager.install(std::slice::from_ref(&options.old_package)) {
                    output::error(&format!("Failed to restore: {}", e2));
                    output::warning("Manual intervention may be required!");
                    return Err(DeclarchError::Other(format!(
                        "Transition failed: {} - Manual cleanup required",
                        e
                    )));
                }
            }

            Err(DeclarchError::Other(format!(
                "Transition failed, changes rolled back: {}",
                e
            )))
        }
    }
}

/// Discover the actual AUR package name for a given package
/// This handles cases where the config name differs from the actual AUR package name
/// (e.g., config says "hyprland" but AUR package is "hyprland-git")
fn discover_aur_package_name(package_name: &str) -> Option<String> {
    // Query pacman -Qi to get package info
    let output = Command::new("pacman")
        .args(["-Qi", package_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;

    // Parse the "Name" field to get the actual package name
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name") {
            if let Some(name) = line.split(':').nth(1) {
                let actual_name = name.trim();
                // Return the actual name only if it differs from config name
                if actual_name != package_name {
                    return Some(actual_name.to_string());
                }
            }
            break;
        }
    }

    None
}

fn determine_backend(package_name: &str, backend_opt: Option<String>) -> Result<Backend> {
    if let Some(backend_str) = backend_opt {
        match backend_str.to_lowercase().as_str() {
            "aur" => Ok(Backend::Aur),
            "flatpak" => Ok(Backend::Flatpak),
            "soar" => Ok(Backend::Soar),
            _ => Err(DeclarchError::Other(format!(
                "Unknown backend: {}. Use 'aur', 'flatpak', or 'soar'",
                backend_str
            ))),
        }
    } else {
        // Auto-detect based on prefix
        if package_name.starts_with("flatpak:") {
            Ok(Backend::Flatpak)
        } else if package_name.starts_with("soar:") {
            Ok(Backend::Soar)
        } else {
            // Default to AUR
            Ok(Backend::Aur)
        }
    }
}

fn execute_transition(
    old_package: &str,
    new_package: &str,
    _backend: &Backend,
    manager: &dyn PackageManager,
) -> Result<()> {
    output::indent(&format!("Uninstalling {}...", old_package.yellow()), 0);

    // Uninstall old package
    manager
        .remove(&[old_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to uninstall {}: {}", old_package, e)))?;

    output::success(&format!("Uninstalled {}", old_package));

    // Install new package
    output::indent(&format!("Installing {}...", new_package.green()), 0);

    manager
        .install(&[new_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to install {}: {}", new_package, e)))?;

    output::success(&format!("Installed {}", new_package));

    Ok(())
}
