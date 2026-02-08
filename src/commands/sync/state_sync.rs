//! State updates after sync operations
//!
//! Updates state.json with new package information.

use crate::core::{resolver, types::{PackageId, PackageMetadata}};
use crate::error::Result;
use crate::state::types::{State, PackageState};
use super::{InstalledSnapshot, SyncOptions};
use chrono::Utc;

/// Update state based on transaction execution results
pub fn update_state(
    state: &State,
    transaction: &resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
    options: &SyncOptions,
) -> Result<State> {
    let mut state = state.clone();

    // Update last_update timestamp if --update was used
    if options.update {
        state.meta.last_update = Some(Utc::now());
    }

    // Collect all packages to upsert
    let packages_to_upsert = transaction
        .to_install
        .iter()
        .chain(transaction.to_adopt.iter())
        .chain(transaction.to_update_project_metadata.iter());

    // Track failed installations (packages not found in snapshot)
    let mut failed_packages = Vec::new();

    // Upsert packages into state
    for pkg in packages_to_upsert {
        let meta = find_package_metadata(pkg, installed_snapshot);

        // Only add to state if package is actually installed
        if meta.is_none() {
            failed_packages.push(pkg.clone());
            continue;
        }

        let version = meta.and_then(|m| m.version.clone());
        let key = resolver::make_state_key(pkg);

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                aur_package_name: None, // Deprecated field, kept for compatibility
                installed_at: Utc::now(),
                version,
            },
        );
    }

    // Remove pruned packages from state
    for pkg in &transaction.to_prune {
        let key = resolver::make_state_key(pkg);
        state.packages.remove(&key);
    }

    Ok(state)
}

/// Find metadata for a package using smart matching
fn find_package_metadata<'a>(
    pkg: &PackageId,
    installed_snapshot: &'a InstalledSnapshot,
) -> Option<&'a PackageMetadata> {
    // Try exact match first
    if let Some(meta) = installed_snapshot.get(pkg) {
        return Some(meta);
    }

    // Use PackageMatcher for smart matching
    let matcher = crate::core::matcher::PackageMatcher::new();
    let matched_id = matcher.find_package(pkg, installed_snapshot)?;
    installed_snapshot.get(&matched_id)
}
