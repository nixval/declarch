//! Upgrade command
//!
//! Upgrades packages to latest versions across all configured backends,
//! then automatically syncs to adopt the changes into state.

use crate::backends::load_all_backends;
use crate::config::types::GlobalConfig;
use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::create_manager;
use crate::ui as output;
use rayon::prelude::*;

pub struct UpgradeOptions {
    /// Target specific backends (None = all backends)
    pub backends: Option<Vec<String>>,
    /// Skip automatic sync after upgrade
    pub no_sync: bool,
}

/// Run upgrade for configured backends
pub fn run(options: UpgradeOptions) -> Result<()> {
    output::separator();
    output::info("Loading backend configurations...");

    // Load all configured backends
    let all_backends = load_all_backends()?;

    if all_backends.is_empty() {
        output::warning("No backends configured");
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Filter backends if specific ones requested
    let backends_to_upgrade: Vec<(String, _)> = match &options.backends {
        Some(target_backends) => all_backends
            .into_iter()
            .filter(|(name, _)| target_backends.contains(name))
            .collect(),
        None => all_backends.into_iter().collect(),
    };

    if backends_to_upgrade.is_empty() {
        output::warning("No matching backends found");
        return Ok(());
    }

    let global_config = GlobalConfig::default();

    // Upgrade backends in parallel for better performance
    output::info("Upgrading packages...");
    output::separator();

    let upgrade_results: Vec<(String, bool)> = backends_to_upgrade
        .par_iter()
        .filter_map(|(name, config)| {
            // Check if backend has upgrade_cmd configured
            if config.upgrade_cmd.is_none() {
                output::info(&format!(
                    "Skipping '{}': no upgrade_cmd configured",
                    name
                ));
                return Some((name.clone(), false));
            }

            // Create manager for this backend
            match create_manager(
                &Backend::from(name.as_str()),
                &global_config,
                false,
            ) {
                Ok(manager) => {
                    if !manager.is_available() {
                        output::warning(&format!("Backend '{}' is not available", name));
                        return Some((name.clone(), false));
                    }

                    if !manager.supports_upgrade() {
                        output::warning(&format!(
                            "Backend '{}' does not support upgrade",
                            name
                        ));
                        return Some((name.clone(), false));
                    }

                    match manager.upgrade() {
                        Ok(()) => {
                            Some((name.clone(), true))
                        }
                        Err(e) => {
                            output::warning(&format!(
                                "Failed to upgrade '{}': {}",
                                name, e
                            ));
                            Some((name.clone(), false))
                        }
                    }
                }
                Err(e) => {
                    output::warning(&format!(
                        "Failed to create manager for '{}': {}",
                        name, e
                    ));
                    Some((name.clone(), false))
                }
            }
        })
        .collect();

    // Count results
    let upgraded_count = upgrade_results.iter().filter(|(_, success)| *success).count();
    let skipped_count = upgrade_results.len() - upgraded_count;

    output::separator();
    if upgraded_count > 0 {
        output::success(&format!("Upgraded {} backend(s)", upgraded_count));
    }
    if skipped_count > 0 {
        output::info(&format!("Skipped {} backend(s)", skipped_count));
    }

    // Auto-sync after upgrade (unless --no-sync)
    if !options.no_sync && upgraded_count > 0 {
        output::separator();
        output::info("Running sync to adopt upgraded packages...");
        
        // Run sync with adopt mode to update state with new versions
        crate::commands::sync::run(crate::commands::sync::SyncOptions {
            dry_run: false,
            prune: false,
            gc: false,
            update: false,
            yes: true,  // Auto-yes since we just did upgrade
            force: false,
            target: None,
            noconfirm: false,
            hooks: false,
            modules: Vec::new(),
        })?;
        
        output::separator();
        output::success("Upgrade and sync completed successfully!");
    } else if options.no_sync {
        output::info("Skipping sync (--no-sync flag set)");
        output::warning("State may be out of sync with actual installed packages");
        output::info("Run 'declarch sync' manually to fix state");
    }

    Ok(())
}
