//! State updates after sync operations
//!
//! Updates state.json with new package information.

use crate::core::{resolver, types::{Backend, PackageId, PackageMetadata}};
use crate::error::Result;
use crate::state::types::{State, PackageState};
use super::{InstalledSnapshot, SyncOptions};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Command;

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

        // CRITICAL FIX: Only add to state if package is actually installed
        // Packages that failed installation won't be in the snapshot
        if meta.is_none() {
            // Package failed to install - track it but don't add to state
            failed_packages.push(pkg.clone());
            continue;
        }

        let version = meta.and_then(|m| m.version.clone());
        let key = resolver::make_state_key(pkg);

        // Discover actual AUR package name if applicable
        let aur_package_name = if pkg.backend == Backend::Aur {
            discover_aur_package_name(&pkg.name)
        } else {
            None
        };

        state.packages.insert(
            key,
            PackageState {
                backend: pkg.backend.clone(),
                config_name: pkg.name.clone(),
                provides_name: pkg.name.clone(),
                aur_package_name,
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
