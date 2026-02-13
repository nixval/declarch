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
    /// Verbose output
    pub verbose: bool,
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

    // First pass: check which backends can be upgraded
    let mut upgradable_backends = Vec::new();
    let mut skipped_no_cmd = Vec::new();
    let mut skipped_not_available = Vec::new();

    for (name, config) in backends_to_upgrade {
        if config.upgrade_cmd.is_none() {
            skipped_no_cmd.push(name);
            continue;
        }
        
        match create_manager(&Backend::from(name.as_str()), &global_config, false) {
            Ok(manager) => {
                if manager.is_available() && manager.supports_upgrade() {
                    upgradable_backends.push((name, manager));
                } else if !manager.is_available() {
                    skipped_not_available.push(name);
                }
            }
            Err(_) => {
                skipped_not_available.push(name);
            }
        }
    }

    // Show compact summary of skipped backends
    if !skipped_no_cmd.is_empty() {
        output::warning(&format!(
            "Skipped (no upgrade_cmd): {}",
            skipped_no_cmd.join(", ")
        ));
    }
    if !skipped_not_available.is_empty() {
        output::warning(&format!(
            "Skipped (not available): {}",
            skipped_not_available.join(", ")
        ));
    }

    if upgradable_backends.is_empty() {
        output::info("No backends to upgrade");
        return Ok(());
    }

    // Upgrade backends in parallel
    output::info("Upgrading packages...");
    output::separator();

    let upgrade_results: Vec<(String, bool)> = upgradable_backends
        .into_par_iter()
        .map(|(name, manager)| {
            match manager.upgrade() {
                Ok(()) => (name, true),
                Err(e) => {
                    output::warning(&format!("Failed to upgrade '{}': {}", name, e));
                    (name, false)
                }
            }
        })
        .collect();

    // Count results
    let upgraded_count = upgrade_results.iter().filter(|(_, success)| *success).count();
    let failed_count = upgrade_results.len() - upgraded_count;

    output::separator();
    if upgraded_count > 0 {
        output::success(&format!("Upgraded {} backend(s)", upgraded_count));
    }
    if failed_count > 0 {
        output::warning(&format!("Failed {} backend(s)", failed_count));
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
            diff: false,
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
