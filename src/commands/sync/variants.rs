//! Variant detection and resolution
//!
//! Handles package variant matching (-bin, -git, etc.) for smart package identification

use super::InstalledSnapshot;
use crate::core::types::PackageId;

/// Common package variant suffixes for smart matching
const VARIANT_SUFFIXES: &[&str] = &[
    "-bin",
    "-git",
    "-hg",
    "-nightly",
    "-beta",
    "-wayland",
    "-appimage",
];

/// Try to find a package variant in the installed snapshot
/// Returns the variant name if found, otherwise None
///
/// This works for any backend, not just AUR. For example:
/// - "hyprland" might match "hyprland-git" or "hyprland-bin"
/// - "firefox" might match "firefox-nightly"
pub fn find_variant(
    package_name: &str,
    backend: &str,
    installed_snapshot: &InstalledSnapshot,
) -> Option<String> {
    // Try each suffix variant
    for suffix in VARIANT_SUFFIXES {
        let alt_name = format!("{}{}", package_name, suffix);
        let alt_id = PackageId {
            name: alt_name.clone(),
            backend: crate::core::types::Backend::from(backend),
        };
        if installed_snapshot.contains_key(&alt_id) {
            return Some(alt_name);
        }
    }

    // Try prefix match (e.g., "hyprland-git" → "hyprland")
    if let Some((prefix, _)) = package_name.split_once('-') {
        let alt_id = PackageId {
            name: prefix.to_string(),
            backend: crate::core::types::Backend::from(backend),
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

    // Try variant matching for the same backend
    if let Some(variant) = find_variant(&pkg.name, pkg.backend.name(), installed_snapshot) {
        return variant;
    }

    // Try case-insensitive substring match for flatpak-style backends
    // (where package IDs are long like com.spotify.Client)
    let search = pkg.name.to_lowercase();
    for installed_id in installed_snapshot.keys() {
        if installed_id.backend == pkg.backend && installed_id.name.to_lowercase().contains(&search)
        {
            return installed_id.name.clone();
        }
    }

    // No match found - return original name
    pkg.name.clone()
}

/// Check if a package name looks like a variant (has known suffix)
#[allow(dead_code)]
pub fn is_variant(package_name: &str) -> bool {
    VARIANT_SUFFIXES
        .iter()
        .any(|suffix| package_name.ends_with(suffix))
}
