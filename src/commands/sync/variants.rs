//! Variant detection and resolution
//!
//! Handles AUR package variant matching (-bin, -git, etc.)

use super::InstalledSnapshot;
use crate::constants::AUR_SUFFIXES;
use crate::core::types::{Backend, PackageId};

/// Try to find an AUR package variant in the installed snapshot
/// Returns the variant name if found, otherwise None
pub fn find_aur_variant(
    package_name: &str,
    installed_snapshot: &InstalledSnapshot,
) -> Option<String> {
    // Try each suffix variant
    for suffix in AUR_SUFFIXES {
        let alt_name = format!("{}{}", package_name, suffix);
        let alt_id = PackageId {
            name: alt_name.clone(),
            backend: Backend::Aur,
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(alt_name);
        }
    }

    // Try prefix match (e.g., "hyprland-git" → "hyprland")
    if let Some((prefix, _)) = package_name.split_once('-') {
        let alt_id = PackageId {
            name: prefix.to_string(),
            backend: Backend::Aur,
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(prefix.to_string());
        }
    }

    None
}

/// Smart matching: Find the actual installed package name for a config package
/// This handles variant matching (e.g., "hyprland" → "hyprland-git")
pub fn resolve_installed_package_name(
    pkg: &PackageId,
    installed_snapshot: &InstalledSnapshot,
) -> String {
    // Try exact match first
    if installed_snapshot.contains_key(pkg) {
        return pkg.name.clone();
    }

    // Try smart match based on backend
    match pkg.backend {
        Backend::Aur => {
            // Use helper function for variant matching
            if let Some(variant) = find_aur_variant(&pkg.name, installed_snapshot) {
                return variant;
            }
            pkg.name.clone()
        }
        Backend::Flatpak => {
            let search = pkg.name.to_lowercase();
            for installed_id in installed_snapshot.keys() {
                if installed_id.backend == Backend::Flatpak
                    && installed_id.name.to_lowercase().contains(&search)
                {
                    return installed_id.name.clone();
                }
            }
            pkg.name.clone()
        }
        Backend::Soar
        | Backend::Npm
        | Backend::Yarn
        | Backend::Pnpm
        | Backend::Bun
        | Backend::Pip
        | Backend::Cargo
        | Backend::Brew
        | Backend::Custom(_) => {
            // These backends require exact matching - no smart matching needed
            pkg.name.clone()
        }
    }
}
