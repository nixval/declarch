//! Package Matching Module
//!
//! Smart matching logic to find packages across variants and naming schemes.
//!
//! Works with any backend - variant detection is generic, not AUR-specific.

use crate::core::types::{PackageId, PackageMetadata};
use std::collections::HashMap;

/// Common package variant suffixes
/// These apply to any backend that supports variants (not just AUR)
const VARIANT_SUFFIXES: &[&str] = &[
    "-bin", "-git", "-hg", "-nightly", "-beta", "-wayland", "-fs", "-alpha", "-rc", "-pre",
];

/// Smart matcher for finding packages across variants
pub struct PackageMatcher;

impl PackageMatcher {
    /// Create new matcher
    pub fn new() -> Self {
        Self
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

        // Strategy 2: Variant matching (suffix/prefix) - works for any backend
        if let Some(variant) = self.find_variant_match(target, installed_snapshot) {
            return Some(variant);
        }

        // Strategy 3: Fuzzy/substring matching - useful for long package IDs (flatpak-style)
        if let Some(fuzzy) = self.find_fuzzy_match(target, installed_snapshot) {
            return Some(fuzzy);
        }

        None
    }

    /// Find variant match using suffix and prefix matching
    /// Works for any backend, not just AUR
    fn find_variant_match(
        &self,
        target: &PackageId,
        installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    ) -> Option<PackageId> {
        let backend = &target.backend;

        // Strategy A: Try adding suffixes to target name
        for suffix in VARIANT_SUFFIXES {
            let alt_name = format!("{}{}", target.name, suffix);
            let alt_id = PackageId {
                name: alt_name,
                backend: backend.clone(),
            };
            if installed_snapshot.contains_key(&alt_id) {
                return Some(alt_id);
            }
        }

        // Strategy B: Try stripping suffixes from target name
        for suffix in VARIANT_SUFFIXES {
            if let Some(base) = target.name.strip_suffix(suffix) {
                let alt_id = PackageId {
                    name: base.to_string(),
                    backend: backend.clone(),
                };
                if installed_snapshot.contains_key(&alt_id) {
                    return Some(alt_id);
                }
            }
        }

        None
    }

    /// Find fuzzy match using substring matching
    /// Useful for backends with long package IDs (e.g., com.spotify.Client)
    fn find_fuzzy_match(
        &self,
        target: &PackageId,
        installed_snapshot: &HashMap<PackageId, PackageMetadata>,
    ) -> Option<PackageId> {
        let search = target.name.to_lowercase();
        let backend = &target.backend;

        for installed_id in installed_snapshot.keys() {
            // Only match within same backend
            if &installed_id.backend != backend {
                continue;
            }

            let installed_name = installed_id.name.to_lowercase();
            
            // Check if one contains the other
            if installed_name.contains(&search) || search.contains(&installed_name) {
                return Some(installed_id.clone());
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

        // Check if one is variant of the other
        self.is_variant_match(pkg1, pkg2)
    }

    /// Check if two package names are variants of each other
    fn is_variant_match(&self, pkg1: &PackageId, pkg2: &PackageId) -> bool {
        // Strip suffixes from both and compare
        let base1 = self.strip_suffix(&pkg1.name);
        let base2 = self.strip_suffix(&pkg2.name);

        base1 == base2
    }

    /// Strip known suffixes from package name
    fn strip_suffix(&self, name: &str) -> String {
        for suffix in VARIANT_SUFFIXES {
            if let Some(base) = name.strip_suffix(suffix) {
                return base.to_string();
            }
        }
        name.to_string()
    }

    /// Get all variant names for a package
    pub fn get_variants(&self, base_name: &str) -> Vec<String> {
        let mut variants = vec![base_name.to_string()];

        for suffix in VARIANT_SUFFIXES {
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
    use crate::core::types::Backend;
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
            backend: Backend::from("aur"),
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
            backend: Backend::from("aur"),
        };
        snapshot.insert(git_pkg.clone(), mock_metadata());

        let target = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::from("aur"),
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
            backend: Backend::from("aur"),
        };
        snapshot.insert(base_pkg.clone(), mock_metadata());

        let target = PackageId {
            name: "hyprland-git".to_string(),
            backend: Backend::from("aur"),
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
            backend: Backend::from("aur"),
        };
        let pkg2 = PackageId {
            name: "hyprland-git".to_string(),
            backend: Backend::from("aur"),
        };

        assert!(matcher.is_same_package(&pkg1, &pkg2));
    }

    #[test]
    fn test_is_different_package() {
        let matcher = PackageMatcher::new();

        let pkg1 = PackageId {
            name: "hyprland".to_string(),
            backend: Backend::from("aur"),
        };
        let pkg2 = PackageId {
            name: "wayland".to_string(),
            backend: Backend::from("aur"),
        };

        assert!(!matcher.is_same_package(&pkg1, &pkg2));
    }

    #[test]
    fn test_variant_detection_works_for_any_backend() {
        let matcher = PackageMatcher::new();
        let mut snapshot = HashMap::new();

        // Test with a custom backend (not just aur)
        let git_pkg = PackageId {
            name: "myapp-git".to_string(),
            backend: Backend::from("custom"),
        };
        snapshot.insert(git_pkg.clone(), mock_metadata());

        let target = PackageId {
            name: "myapp".to_string(),
            backend: Backend::from("custom"),
        };

        let result = matcher.find_package(&target, &snapshot);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "myapp-git");
    }
}
