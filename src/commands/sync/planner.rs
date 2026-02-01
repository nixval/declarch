//! Transaction planning for sync operations
//!
//! Determines what packages to install, adopt, prune, and update.

use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{resolver, types::SyncTarget};
use crate::error::{DeclarchError, Result};
use crate::state::types::State;
use crate::ui as output;
use colored::Colorize;
use chrono::Utc;
use super::{InstalledSnapshot, SyncOptions, ManagerMap};

/// Create transaction from current state and desired config
/// This is a wrapper that calls resolve_and_filter_packages
pub fn create_transaction(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    resolve_and_filter_packages(config, state, installed_snapshot, managers, sync_target)
}

/// Resolve transaction and filter packages by available backends
pub fn resolve_and_filter_packages(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    use crate::core::types::Backend;
    use std::collections::HashMap;

    // Filter packages to only include available backends
    let available_backends: std::collections::HashSet<Backend> = managers.keys().cloned().collect();
    let total_packages = config.packages.len();

    // Create filtered packages map
    let filtered_packages: HashMap<_, _> = config
        .packages
        .iter()
        .filter(|(pkg_id, _)| available_backends.contains(&pkg_id.backend))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // Warn about packages from unavailable backends
    let skipped_count = total_packages - filtered_packages.len();
    if skipped_count > 0 {
        output::warning(&format!(
            "Skipping {} package(s) from unavailable backends.",
            skipped_count
        ));

        // Show which packages were skipped
        for (pkg_id, _) in config.packages.iter() {
            if !available_backends.contains(&pkg_id.backend) {
                output::info(&format!(
                    "  Skipping {} (backend '{}' not available)",
                    pkg_id.name, pkg_id.backend
                ));
            }
        }
    }

    // Temporarily replace packages in config with filtered version for resolver
    let original_packages = std::mem::replace(&mut config.packages, filtered_packages);

    let tx = resolver::resolve(config, state, installed_snapshot, sync_target)?;

    // Restore original packages after resolve
    config.packages = original_packages;

    Ok(tx)
}

/// Check for variant package mismatches between config and installed state
pub fn check_variant_transitions(
    config: &loader::MergedConfig,
    installed_snapshot: &InstalledSnapshot,
    state: &State,
    tx: &resolver::Transaction,
    sync_target: &SyncTarget,
    options: &SyncOptions,
) -> Result<()> {
    use crate::core::matcher::PackageMatcher;
    use crate::core::types::Backend;
    use std::collections::HashSet;

    let matcher = PackageMatcher::new();
    let mut variant_mismatches: Vec<(String, String)> = Vec::new();

    // Only check for variant transitions in full sync or when targeting specific backends
    if matches!(sync_target, SyncTarget::All | SyncTarget::Backend(_)) {
        // Re-filter packages for variant checking
        let available_backends: HashSet<Backend> = installed_snapshot
            .keys()
            .map(|pkg_id| pkg_id.backend.clone())
            .collect();

        for pkg_id in config
            .packages
            .keys()
            .filter(|pkg_id| available_backends.contains(&pkg_id.backend))
        {
            // Skip if this package is already in transaction to install
            if tx.to_install.iter().any(|p| p.name == pkg_id.name) {
                continue;
            }

            // Check if there's a variant of this package installed
            if let Some(matched_id) = matcher.find_package(pkg_id, installed_snapshot) {
                // If matched name is different from config name, it's a variant
                if matched_id.name != pkg_id.name {
                    // Check if this variant is NOT already tracked in state
                    let state_key = resolver::make_state_key(pkg_id);
                    let state_pkg = state.packages.get(&state_key);

                    // Only report if not tracked (means user might have manually changed it)
                    if state_pkg.is_none()
                        || state_pkg
                            .and_then(|s| s.aur_package_name.as_ref())
                            .map(|n| n != &matched_id.name)
                            .unwrap_or(false)
                    {
                        variant_mismatches.push((pkg_id.name.clone(), matched_id.name));
                    }
                }
            }
        }
    }

    // If variant mismatches found, error with helpful message
    if !variant_mismatches.is_empty() && !options.force {
        output::separator();
        output::error("Variant transition detected!");
        println!("\nThe following packages have different variants installed:\n");

        for (config_name, installed_name) in &variant_mismatches {
            println!(
                "  {}  →  {}",
                config_name.cyan().bold(),
                installed_name.yellow().bold()
            );
        }

        println!(
            "\n{}",
            "This requires explicit transition to avoid unintended changes.".dimmed()
        );
        println!("\n{}", "To resolve this:".bold());
        println!("  1. For each package, run:");
        for (config_name, installed_name) in &variant_mismatches {
            println!(
                "     {}",
                format!(
                    "declarch switch {} {}",
                    installed_name.yellow(),
                    config_name.cyan()
                )
                .bold()
            );
        }
        println!("\n  2. Or, update your config to match the installed variant");
        println!(
            "\n  3. Use {} to bypass this check (not recommended)",
            "--force".yellow().bold()
        );

        return Err(DeclarchError::Other(
            "Variant transition required. Use 'declarch switch' or update your config.".to_string(),
        ));
    }

    Ok(())
}

/// Warn about partial upgrades when system hasn't been updated recently
pub fn warn_partial_upgrade(state: &State, tx: &resolver::Transaction, options: &SyncOptions) {
    if !options.update && !tx.to_install.is_empty() {
        let should_warn = match state.meta.last_update {
            Some(last) => Utc::now().signed_duration_since(last).num_hours() > 24,
            None => true,
        };

        if should_warn {
            let time_str = state
                .meta
                .last_update
                .map(|t| format!("{}h ago", Utc::now().signed_duration_since(t).num_hours()))
                .unwrap_or("unknown".to_string());

            output::separator();
            println!(
                "{} Last system update: {}. Use {} to refresh.",
                "⚠ Partial Upgrade Risk:".yellow().bold(),
                time_str.white(),
                "--update".bold()
            );
        }
    }
}

/// Display the transaction plan to the user
pub fn display_transaction_plan(
    tx: &resolver::Transaction,
    should_prune: bool,
) {
    // Display in compact format
    if !tx.to_install.is_empty() || !tx.to_adopt.is_empty() || (!tx.to_prune.is_empty() && should_prune) {
        println!("{}", "Changes:".green().bold());

        if !tx.to_install.is_empty() {
            println!(
                "  Install: {}",
                tx.to_install
                    .iter()
                    .map(|p| format!("{} ({})", p.name, p.backend))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !tx.to_adopt.is_empty() {
            println!(
                "  Adopt:   {}",
                tx.to_adopt
                    .iter()
                    .map(|p| format!("{} ({})", p.name, p.backend))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !tx.to_prune.is_empty() && should_prune {
            println!(
                "  Remove:  {}",
                tx.to_prune
                    .iter()
                    .map(|p| {
                        if CRITICAL_PACKAGES.contains(&p.name.as_str()) {
                            format!("{} ({}) [keep]", p.name, p.backend)
                        } else {
                            format!("{} ({})", p.name, p.backend)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
