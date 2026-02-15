//! State updates after sync operations
//!
//! Updates state.json with new package information.

use super::{InstalledSnapshot, SyncOptions};
use crate::core::{
    resolver,
    types::{PackageId, PackageMetadata},
};
use crate::error::Result;
use crate::state::types::{PackageState, State};
use crate::ui;
use chrono::Utc;
use std::collections::HashSet;

/// Update state based on transaction execution results
pub fn update_state(
    state: &State,
    transaction: &resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
    options: &SyncOptions,
) -> Result<State> {
    // For backwards compatibility, call new function with empty success list
    update_state_with_success(state, transaction, installed_snapshot, options, &[])
}

/// Update state with knowledge of which packages successfully installed
pub fn update_state_with_success(
    state: &State,
    transaction: &resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
    options: &SyncOptions,
    successfully_installed: &[PackageId],
) -> Result<State> {
    let mut state = state.clone();

    // Update last_update timestamp if --update was used
    if options.update {
        state.meta.last_update = Some(Utc::now());
    }

    // Create a set of successfully installed packages for fast lookup
    let success_set: HashSet<&PackageId> = successfully_installed.iter().collect();

    // Track installation results
    let mut added_count = 0;
    let mut failed_count = 0;

    // Process to_install - only add successful ones
    for pkg in &transaction.to_install {
        if !success_set.contains(pkg) {
            failed_count += 1;
            continue;
        }

        let (version, actual_name) =
            if let Some((meta, name)) = find_package_info(pkg, installed_snapshot) {
                (meta.version.clone(), name)
            } else {
                (None, None)
            };
        let key = resolver::make_state_key(pkg);

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                actual_package_name: actual_name,
                installed_at: Utc::now(),
                version,
            },
        );
        added_count += 1;
    }

    // Process adoptions (these are already installed, so always add)
    for pkg in &transaction.to_adopt {
        let (version, actual_name) =
            if let Some((meta, name)) = find_package_info(pkg, installed_snapshot) {
                (meta.version.clone(), name)
            } else {
                (None, None)
            };
        let key = resolver::make_state_key(pkg);

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                actual_package_name: actual_name,
                installed_at: Utc::now(),
                version,
            },
        );
        added_count += 1;
    }

    // Remove pruned packages from state
    for pkg in &transaction.to_prune {
        let key = resolver::make_state_key(pkg);
        state.packages.remove(&key);
    }

    // Report results
    if added_count > 0 {
        ui::success(&format!("Added {} package(s) to state", added_count));
    }
    if failed_count > 0 {
        ui::warning(&format!(
            "{} package(s) failed and were not added to state",
            failed_count
        ));
    }

    Ok(state)
}

/// Find package info using smart matching
/// Returns (metadata, actual_package_name) where actual_package_name is the
/// real name from system (may differ from config name for variants)
fn find_package_info<'a>(
    pkg: &PackageId,
    installed_snapshot: &'a InstalledSnapshot,
) -> Option<(&'a PackageMetadata, Option<String>)> {
    // Try exact match first
    if let Some(meta) = installed_snapshot.get(pkg) {
        return Some((meta, None)); // No variant, actual == config
    }

    // Use PackageMatcher for smart matching (variant detection)
    let matcher = crate::core::matcher::PackageMatcher::new();
    let matched_id = matcher.find_package(pkg, installed_snapshot)?;
    let meta = installed_snapshot.get(&matched_id)?;

    // Return metadata + actual package name (variant)
    Some((meta, Some(matched_id.name.clone())))
}
