//! Upgrade command
//!
//! Upgrades packages to latest versions across all configured backends,
//! then automatically syncs to adopt the changes into state.

use crate::backends::load_all_backends_unified;
use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::PackageManager;
use crate::ui as output;
use std::collections::HashSet;

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

    // Load all configured backends (import-based or legacy)
    let all_backends = load_all_backends_unified()?;

    if all_backends.is_empty() {
        output::warning("No backends configured");
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Filter backends if specific ones requested
    let mut backends_to_upgrade: Vec<(String, _)> = match &options.backends {
        Some(target_backends) => {
            let target_set: HashSet<_> = target_backends.iter().cloned().collect();
            let selected: Vec<_> = all_backends
                .into_iter()
                .filter(|(name, _)| target_set.contains(name))
                .collect();

            let selected_names: HashSet<_> =
                selected.iter().map(|(name, _)| name.clone()).collect();
            let mut unknown: Vec<_> = target_set
                .into_iter()
                .filter(|name| !selected_names.contains(name))
                .collect();
            unknown.sort();
            if !unknown.is_empty() {
                output::warning(&format!("Unknown backend(s): {}", unknown.join(", ")));
            }

            selected
        }
        None => all_backends.into_iter().collect(),
    };
    backends_to_upgrade.sort_by(|a, b| a.0.cmp(&b.0));

    if backends_to_upgrade.is_empty() {
        output::warning("No matching backends found");
        return Ok(());
    }

    let runtime_config = load_runtime_config_for_command("upgrade command");

    // First pass: check which backends can be upgraded
    let mut upgradable_backends = Vec::new();
    let mut skipped_no_cmd = Vec::new();
    let mut skipped_not_available = Vec::new();

    for (name, mut config) in backends_to_upgrade {
        apply_runtime_backend_overrides(&mut config, &name, &runtime_config);

        if !crate::utils::platform::backend_supports_current_os(&config) {
            output::warning(&format!(
                "Skipping '{}' on this device (not for current OS).",
                name
            ));
            continue;
        }

        if config.upgrade_cmd.is_none() {
            skipped_no_cmd.push(name);
            continue;
        }

        let manager: Box<dyn PackageManager> =
            Box::new(crate::backends::GenericManager::from_config(
                config,
                Backend::from(name.as_str()),
                false,
            ));
        if manager.is_available() && manager.supports_upgrade() {
            upgradable_backends.push((name, manager));
        } else if !manager.is_available() {
            skipped_not_available.push(name);
        } else if options.verbose {
            output::warning(&format!("Skipped '{}': upgrade not supported", name));
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

    // Upgrade backends sequentially to avoid interactive TTY conflicts.
    output::info("Upgrading packages...");
    output::separator();

    let mut upgrade_results = Vec::new();
    for (name, manager) in upgradable_backends {
        match manager.upgrade() {
            Ok(()) => upgrade_results.push((name, true)),
            Err(e) => {
                output::warning(&format!("Failed to upgrade '{}': {}", name, e));
                upgrade_results.push((name, false));
            }
        }
    }

    // Count results
    let upgraded_count = upgrade_results
        .iter()
        .filter(|(_, success)| *success)
        .count();
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
            yes: true, // Auto-yes since we just did upgrade
            force: false,
            target: None,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: Vec::new(),
            diff: false,
            format: None,
            output_version: None,
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
