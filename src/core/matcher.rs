//! Package Matching Module
//!
//! Smart matching logic to find packages across variants and naming schemes.
//!
//! # Problem
//!
//! In Arch/AUR, packages can have multiple variants:
//! - hyprland, hyprland-git, hyprland-bin
//! - pipewire, pipewire-full, pipewire-jack2
//!
//! Users specify the base name, but the system might have a variant installed.
//!
//! # Solution
//!
//! Smart matching strategies:
//! 1. Exact match
//! 2. Suffix matching (try common variants)
//! 3. Prefix matching (strip suffixes)
//! 4. Fuzzy matching (for Flatpak)

use crate::constants::AUR_SUFFIXES;
use crate::core::types::{Backend, PackageId, PackageMetadata};
use std::collections::HashMap;

/// Smart matcher for finding packages across variants
pub struct PackageMatcher {
    // Future: add configurable suffixes, aliases, etc.
}

impl PackageMatcher {
    /// Create new matcher
    pub fn new() -> Self {
        Self {}
    }

    /// Find a package in the installed snapshot using smart matching
    ///
    /// Returns the PackageId of the best match (cloned), or None if not found
    pub fn find_package(
        &self,
        target: &PackageId,
        installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    ) -> Option<PackageId> {
        // Strategy 1: Exact match
        if installed_snapshot.contains_key(target) {
            return Some(target.clone());
        }

        match target.backend {
            Backend::Aur => self.find_aur_package(target, installed_snapshot),
            Backend::Flatpak => self.find_flatpak_package(target, installed_snapshot),
            Backend::Soar
            | Backend::Npm
            | Backend::Yarn
            | Backend::Pnpm
            | Backend::Bun
            | Backend::Pip
            | Backend::Cargo
            | Backend::Brew
            | Backend::Custom(_) => {
                // These backends require exact matching (no variants)
                None
            }
        }
    }

    /// Find AUR package using suffix and prefix matching
    fn find_aur_package(
        &self,
        target: &PackageId,
        installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    ) -> Option<PackageId> {
        // Strategy A: Try suffixes
        for suffix in AUR_SUFFIXES {
            let alt_name = format!("{}{}", target.name, suffix);
            let alt_id = PackageId {
                name: alt_name,
                backend: Backend::Aur,
            };
            if installed_snapshot.contains_key(&alt_id) {
                return Some(alt_id);
            }
        }

        // Strategy B: Try prefix (strip suffix from target)
        // Only strip if the suffix is a known variant suffix
        for suffix in AUR_SUFFIXES {
            if let Some(base) = target.name.strip_suffix(suffix) {
                let alt_id = PackageId {
                    name: base.to_string(),
                    backend: Backend::Aur,
                };
                if installed_snapshot.contains_key(&alt_id) {
                    return Some(alt_id);
                }
            }
        }

        None
    }

    /// Find Flatpak package using fuzzy matching
    fn find_flatpak_package(
        &self,
        target: &PackageId,
        installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    ) -> Option<PackageId> {
        // Strategy C: Fuzzy match
        // Config: "spotify" -> System: "com.spotify.Client"
        let search = target.name.to_lowercase();

        for installed_id in installed_snapshot.keys() {
            if installed_id.backend == Backend::Flatpak {
                let installed_name = installed_id.name.to_lowercase();
                if installed_name.contains(&search) {
                    return Some(installed_id.clone());
                }
            }
        }

        None
    }

    /// Check if two package IDs refer to the same package (different variants)
    ///
    /// Example: hyprland and hyprland-git return true
    pub fn is_same_package(&self, pkg1: &PackageId, pkg2: &PackageId) -> bool {
        if pkg1.backend != pkg2.backend {
            return false;
        }

        // Exact match
        if pkg1.name == pkg2.name {
            return true;
        }

        match pkg1.backend {
            Backend::Aur => {
                // Check if one is suffix of the other
                self.is_variant_match(pkg1, pkg2)
            }
            Backend::Flatpak => {
                // Check if one name contains the other
                let name1 = pkg1.name.to_lowercase();
                let name2 = pkg2.name.to_lowercase();
                name1.contains(&name2) || name2.contains(&name1)
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
                // These backends require exact matching
                false
            }
        }
    }

    /// Check if two AUR package names are variants of each other
    fn is_variant_match(&self, pkg1: &PackageId, pkg2: &PackageId) -> bool {
        // Strip suffixes from both and compare
        let base1 = self.strip_suffix(&pkg1.name);
        let base2 = self.strip_suffix(&pkg2.name);

        base1 == base2
    }

    /// Strip known suffixes from package name
    fn strip_suffix(&self, name: &str) -> String {
        for suffix in AUR_SUFFIXES {
            if let Some(base) = name.strip_suffix(suffix) {
                return base.to_string();
            }
        }
        name.to_string()
    }

    /// Get all variant names for a package
    pub fn get_variants(&self, base_name: &str) -> Vec<String> {
        let mut variants = vec![base_name.to_string()];

        for suffix in AUR_SUFFIXES {
            variants.push(format!("{}{}", base_name, suffix));
        }

        variants
    }
}

impl Default for PackageMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_metadata() -> PackageMetadata {
        PackageMetadata {
            version: Some("1.0.0".to_string()),
            variant: None,
            installed_at: Utc::now(),
            source_file: None,
        }
    }

    #[test]
    fn test_exact_match() {
        let matcher = PackageMatcher::new();
        let mut snapshot = HashMap::new();

        let pkg_id = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::Aur,
        };
        snapshot.insert(pkg_id.clone(), mock_metadata());

        let result = matcher.find_package(&pkg_id, &snapshot);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "hyprland");
    }

    #[test]
    fn test_suffix_match() {
        let matcher = PackageMatcher::new();
        let mut snapshot = HashMap::new();

        let git_pkg = PackageId {
            name: "hyprland-git".to_string(),
            backend: Backend::Aur,
        };
        snapshot.insert(git_pkg.clone(), mock_metadata());

        let target = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::Aur,
        };

        let result = matcher.find_package(&target, &snapshot);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "hyprland-git");
    }

    #[test]
    fn test_prefix_match() {
        let matcher = PackageMatcher::new();
        let mut snapshot = HashMap::new();

        let base_pkg = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::Aur,
        };
        snapshot.insert(base_pkg.clone(), mock_metadata());

        let target = PackageId {
            name: "hyprland-git".to_string(),
            backend: Backend::Aur,
        };

        let result = matcher.find_package(&target, &snapshot);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "hyprland");
    }

    #[test]
    fn test_is_same_package() {
        let matcher = PackageMatcher::new();

        let pkg1 = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::Aur,
        };
        let pkg2 = PackageId {
            name: "hyprland-git".to_string(),
            backend: Backend::Aur,
        };

        assert!(matcher.is_same_package(&pkg1, &pkg2));
    }

    #[test]
    fn test_is_different_package() {
        let matcher = PackageMatcher::new();

        let pkg1 = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::Aur,
        };
        let pkg2 = PackageId {
            name: "wayland".to_string(),
            backend: Backend::Aur,
        };

        assert!(!matcher.is_same_package(&pkg1, &pkg2));
    }
}
