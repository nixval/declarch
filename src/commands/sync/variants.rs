//! Variant detection and resolution
//!
//! Handles AUR package variant matching (-bin, -git, etc.)

use crate::core::types::{Backend, PackageId};
use super::InstalledSnapshot;

/// AUR package variant suffixes for smart matching
const AUR_SUFFIXES: &[&str] = &["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];

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
            backend: Backend::from("aur"),
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(alt_name);
        }
    }

    // Try prefix match (e.g., "hyprland-git" → "hyprland")
    if let Some((prefix, _)) = package_name.split_once('-') {
        let alt_id = PackageId {
            name: prefix.to_string(),
            backend: Backend::from("aur"),
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
    if pkg.backend.0 == "aur" {
        // Use helper function for variant matching
        if let Some(variant) = find_aur_variant(&pkg.name, installed_snapshot) {
            return variant;
        }
        pkg.name.clone()
    } else if pkg.backend.0 == "flatpak" {
        let search = pkg.name.to_lowercase();
        for installed_id in installed_snapshot.keys() {
            if installed_id.backend.0 == "flatpak"
                && installed_id.name.to_lowercase().contains(&search)
            {
                return installed_id.name.clone();
            }
        }
        pkg.name.clone()
    } else {
        // All other backends require exact matching - no smart matching needed
        pkg.name.clone()
    }
}
