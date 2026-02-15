//! Transaction planning for sync operations
//!
//! Determines what packages to install, adopt, prune, and update.

use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{resolver, types::SyncTarget};
use crate::error::{DeclarchError, Result};
use crate::state::types::State;
use crate::ui as output;
use chrono::Utc;
use colored::Colorize;

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

    // Warn about packages from unavailable backends (concise format)
    let skipped_count = total_packages - filtered_packages.len();
    if skipped_count > 0 {
        // Group skipped packages by backend for concise output
        let mut skipped_by_backend: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (pkg_id, _) in config.packages.iter() {
            if !available_backends.contains(&pkg_id.backend) {
                *skipped_by_backend
                    .entry(pkg_id.backend.to_string())
                    .or_insert(0) += 1;
            }
        }

        for (backend, count) in skipped_by_backend {
            output::warning(&format!(
                "Skipping {} package(s), backend '{}' not available. Run 'declarch init --backend {}'",
                count, backend, backend
            ));
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
    // Store (config_name, installed_name, backend) for each mismatch
    let mut variant_mismatches: Vec<(String, String, Backend)> = Vec::new();

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
                            .and_then(|s| s.actual_package_name.as_ref())
                            .map(|n| n != &matched_id.name)
                            .unwrap_or(false)
                    {
                        variant_mismatches.push((
                            pkg_id.name.clone(),
                            matched_id.name,
                            pkg_id.backend.clone(),
                        ));
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

        for (config_name, installed_name, backend) in &variant_mismatches {
            println!(
                "  [{}] {}  →  {}",
                backend.to_string().dimmed(),
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
        for (config_name, installed_name, backend) in &variant_mismatches {
            let backend_prefix = format!("{}:", backend);
            println!(
                "     {}",
                format!(
                    "declarch switch {}{} {}{}",
                    backend_prefix.yellow(),
                    installed_name.yellow(),
                    backend_prefix.cyan(),
                    config_name.cyan()
                )
                .bold()
            );
        }
        println!("\n  2. Or, update your config to match the installed variant:");
        for (config_name, installed_name, backend) in &variant_mismatches {
            println!(
                "       pkg {{ {} {{ {} }} }}  (was: {})",
                backend.to_string().cyan(),
                installed_name.yellow(),
                config_name.dimmed()
            );
        }
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

/// Group packages by backend for display
fn group_by_backend(
    packages: &[crate::core::types::PackageId],
) -> std::collections::HashMap<String, Vec<String>> {
    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for pkg in packages {
        groups
            .entry(pkg.backend.to_string())
            .or_default()
            .push(pkg.name.clone());
    }
    // Sort packages within each backend
    for packages in groups.values_mut() {
        packages.sort();
    }
    groups
}

/// Format package groups for display
fn format_backend_groups(groups: &std::collections::HashMap<String, Vec<String>>) -> String {
    let mut backends: Vec<_> = groups.keys().collect();
    backends.sort();

    let mut result = String::new();
    for (i, backend) in backends.iter().enumerate() {
        if let Some(packages) = groups.get(*backend) {
            if i > 0 {
                result.push_str("\n         ");
            }
            result.push_str(&format!(
                "({}): {}",
                backend.cyan(),
                packages.join(", ").dimmed()
            ));
        }
    }
    result
}

/// Display the transaction plan to the user with backend grouping
pub fn display_transaction_plan(tx: &resolver::Transaction, should_prune: bool) {
    let has_changes = !tx.to_install.is_empty()
        || !tx.to_adopt.is_empty()
        || (!tx.to_prune.is_empty() && should_prune);

    if !has_changes {
        return;
    }

    println!("{}", "Changes:".green().bold());

    if !tx.to_install.is_empty() {
        let groups = group_by_backend(&tx.to_install);
        println!(
            "  {} {}",
            "Install:".green(),
            format_backend_groups(&groups)
        );
    }

    if !tx.to_adopt.is_empty() {
        let groups = group_by_backend(&tx.to_adopt);
        println!(
            "  {}   {}",
            "Adopt:".yellow(),
            format_backend_groups(&groups)
        );
    }

    if !tx.to_prune.is_empty() && should_prune {
        let groups = group_by_backend(&tx.to_prune);
        let formatted: std::collections::HashMap<String, Vec<String>> = groups
            .into_iter()
            .map(|(backend, packages)| {
                let modified: Vec<String> = packages
                    .into_iter()
                    .map(|p| {
                        if CRITICAL_PACKAGES.contains(&p.as_str()) {
                            format!("{} [keep]", p)
                        } else {
                            p
                        }
                    })
                    .collect();
                (backend, modified)
            })
            .collect();
        println!(
            "  {}  {}",
            "Remove:".red(),
            format_backend_groups(&formatted)
        );
    }
}

/// Display detailed dry-run simulation
/// Shows what would happen without actually executing
pub fn display_dry_run_details(
    tx: &resolver::Transaction,
    should_prune: bool,
    installed_snapshot: &InstalledSnapshot,
) {
    let has_changes = !tx.to_install.is_empty()
        || !tx.to_adopt.is_empty()
        || (!tx.to_prune.is_empty() && should_prune);

    if !has_changes {
        output::success("Dry-run: No changes needed - everything is up to date!");
        return;
    }

    output::separator();
    println!("{}", "🧪 DRY-RUN SIMULATION".cyan().bold());
    println!(
        "{}",
        "   No changes will be made to your system.\n".dimmed()
    );

    // Summary statistics
    let install_count = tx.to_install.len();
    let adopt_count = tx.to_adopt.len();
    let prune_count = if should_prune { tx.to_prune.len() } else { 0 };

    println!("{}", "Summary:".bold());
    if install_count > 0 {
        println!(
            "  • {} new package(s) to install",
            install_count.to_string().green()
        );
    }
    if adopt_count > 0 {
        println!(
            "  • {} package(s) to adopt into state",
            adopt_count.to_string().yellow()
        );
    }
    if prune_count > 0 {
        println!("  • {} package(s) to remove", prune_count.to_string().red());
    }
    println!();

    // Detailed package list
    if !tx.to_install.is_empty() {
        println!("{}", "Packages to install:".green().bold());
        display_package_groups_detailed(&tx.to_install, installed_snapshot);
    }

    if !tx.to_adopt.is_empty() {
        println!("{}", "\nPackages to adopt:".yellow().bold());
        display_package_groups_detailed(&tx.to_adopt, installed_snapshot);
    }

    if !tx.to_prune.is_empty() && should_prune {
        println!("{}", "\nPackages to remove:".red().bold());
        let groups = group_by_backend(&tx.to_prune);
        for (backend, packages) in groups {
            println!("  {}:", backend.cyan());
            for pkg in packages {
                if CRITICAL_PACKAGES.contains(&pkg.as_str()) {
                    println!(
                        "    • {} {} (protected)",
                        pkg.red(),
                        "[will be kept]".yellow()
                    );
                } else {
                    println!("    • {}", pkg.red());
                }
            }
        }
    }

    // Validation warnings
    println!();
    output::separator();
    println!("{}", "Pre-flight Checks:".bold());

    // Check for packages already installed (would be adopted instead)
    let already_installed: Vec<_> = tx
        .to_install
        .iter()
        .filter(|pkg| {
            installed_snapshot
                .keys()
                .any(|pkg_id| pkg_id.name == pkg.name)
        })
        .collect();

    if !already_installed.is_empty() {
        output::warning(&format!(
            "{} package(s) appear to already be installed but not tracked in state",
            already_installed.len()
        ));
        println!("  They will be 'adopted' into declarch's state management.");
    } else {
        println!("  ✅ No conflicts detected");
    }

    println!();
    output::info("To apply these changes, run without --dry-run flag");
}

/// Display packages grouped by backend with detailed info
fn display_package_groups_detailed(
    packages: &[crate::core::types::PackageId],
    installed_snapshot: &InstalledSnapshot,
) {
    let groups = group_by_backend(packages);
    for (backend, pkg_names) in groups {
        println!("  {}:", backend.cyan());
        for name in pkg_names {
            // Check if this package is already installed (would be variant transition)
            let variant_info = installed_snapshot
                .iter()
                .find(|(pkg_id, _)| pkg_id.name == name && pkg_id.backend.to_string() == backend);

            if let Some((_, meta)) = variant_info {
                if let Some(ref version) = meta.version {
                    println!(
                        "    • {} {}",
                        name.green(),
                        format!("(v{} already installed)", version).dimmed()
                    );
                } else {
                    println!(
                        "    • {} {}",
                        name.green(),
                        "(already installed, untracked)".dimmed()
                    );
                }
            } else {
                println!("    • {}", name.green());
            }
        }
    }
}
