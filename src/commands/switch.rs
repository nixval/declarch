use crate::config::types::GlobalConfig;
use crate::core::types::Backend;
use crate::error::{DeclarchError, Result};
use crate::packages::{PackageManager, create_manager};
use crate::state::{self, types::PackageState};
use crate::ui as output;
use chrono::Utc;
use colored::Colorize;

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

    // Acquire exclusive lock to prevent concurrent state modifications
    let lock = state::io::acquire_lock().map_err(|e| {
        crate::error::DeclarchError::Other(format!(
            "Cannot start switch: {}\n\
             If no other declarch process is running, delete the lock file manually.",
            e
        ))
    })?;

    output::info("Analyzing package transition...");
    output::separator();

    // 1. Load current state
    let mut state = state::io::load_state()?;

    // 2. Determine backend from both packages (must be same)
    let (old_backend, new_backend) = determine_backends(
        &options.old_package,
        &options.new_package,
        options.backend.as_deref(),
    )?;
    
    // Cross-backend switch not supported
    if old_backend != new_backend {
        return Err(DeclarchError::Other(
            "Cross-backend switch is not supported. Both packages must use the same backend.".to_string()
        ));
    }
    let backend = old_backend;

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
            output::success("No reverse dependencies found");
        }
        Err(e) => {
            output::warning(&format!("Could not check reverse dependencies: {}", e));
            output::warning("Proceed with caution");
        }
    }

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

    // Create in-memory backup
    let state_backup = state.clone();
    output::info("Created state backup");

    match execute_transition(
        &options.old_package,
        &options.new_package,
        &backend,
        &*manager,
    ) {
        Ok(()) => {
            output::info("Updating declarch state...");

            // Remove old package from state
            state.packages.remove(&old_state_key);

            // Add new package to state
            let new_state_key = format!("{}:{}", backend, options.new_package);

            let new_pkg_state = PackageState {
                backend: backend.clone(),
                config_name: options.new_package.clone(),
                provides_name: options.new_package.clone(),
                actual_package_name: None, // Actual system package name, if different
                installed_at: Utc::now(),
                version: installed
                    .get(&options.new_package)
                    .and_then(|m| m.version.clone()),
            };

            state.packages.insert(new_state_key, new_pkg_state);

            // Update metadata
            state.meta.last_sync = Utc::now();

            // Save state with file locking
            state::io::save_state_locked(&state, &lock)?;

            output::separator();
            output::success(&format!(
                "Transition complete: {} → {}",
                options.old_package.cyan(),
                options.new_package.green()
            ));

            Ok(())
        }

        Err(e) => {
            output::error(&format!("Transition failed: {}", e));
            output::warning("Rolling back state changes...");

            if let Err(e2) = state::io::save_state_locked(&state_backup, &lock) {
                output::error(&format!("Failed to restore state: {}", e2));
                return Err(DeclarchError::Other(format!(
                    "Transition failed and state rollback failed: {} - {}",
                    e, e2
                )));
            }

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

/// Determine backend(s) for switch operation
/// Returns (old_backend, new_backend) - may be same or different
fn determine_backends(
    old_package: &str,
    new_package: &str,
    backend_opt: Option<&str>,
) -> Result<(Backend, Backend)> {
    // Helper to extract backend from prefixed name
    fn extract_backend(name: &str) -> Option<Backend> {
        name.split_once(':').map(|(b, _)| Backend::from(b))
    }
    
    // Helper to strip backend prefix
    fn strip_prefix(name: &str) -> &str {
        name.split_once(':').map(|(_, n)| n).unwrap_or(name)
    }

    if let Some(backend_str) = backend_opt {
        // --backend flag specified: use same backend for both
        let backend = Backend::from(backend_str);
        Ok((backend.clone(), backend))
    } else {
        // Auto-detect from prefixes
        let old_backend = extract_backend(old_package);
        let new_backend = extract_backend(new_package);
        
        match (old_backend, new_backend) {
            (Some(old), Some(new)) => {
                // Both have prefixes - valid
                Ok((old, new))
            }
            (Some(old), None) => {
                // Only old has prefix - assume same backend for new
                Ok((old.clone(), old))
            }
            (None, Some(new)) => {
                // Only new has prefix - assume same backend for old
                Ok((new.clone(), new))
            }
            (None, None) => {
                // Neither has prefix - show helpful error
                let registry = crate::packages::get_registry();
                let backends = registry.lock()
                    .map(|r| r.available_backends())
                    .unwrap_or_default();
                
                let backend_list = if backends.is_empty() {
                    "No backends configured. Run 'declarch init' first.".to_string()
                } else {
                    format!("Available backends: {}", backends.join(", "))
                };
                
                Err(DeclarchError::Other(format!(
                    "Cannot determine backend.\n\n\
                     Use explicit prefix syntax:\n\
                       declarch switch {}:{} {}:{}\n\n\
                     Or specify backend:\n\
                       declarch switch {} {} --backend <BACKEND>\n\n\
                     {}",
                    backends.first().unwrap_or(&"BACKEND".to_string()), strip_prefix(old_package),
                    backends.first().unwrap_or(&"BACKEND".to_string()), strip_prefix(new_package),
                    strip_prefix(old_package), strip_prefix(new_package),
                    backend_list
                )))
            }
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

    manager
        .remove(&[old_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to uninstall {}: {}", old_package, e)))?;

    output::indent(&format!("Installing {}...", new_package.green()), 0);

    manager
        .install(&[new_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to install {}: {}", new_package, e)))?;

    Ok(())
}
