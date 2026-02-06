//! Package Identity Module
//!
//! Handles package identity resolution, especially for Arch/AUR "provides" relationships
//! and variant detection (e.g., hyprland vs hyprland-git).
//!
//! # Problem
//!
//! In Arch/AUR, a package can provide a different name than its package name:
//! - Package: `pipewire-full` → Provides: `pipewire`
//! - Package: `python-poetry-core` → Provides: `python-poetry`
//!
//! This causes issues when tracking packages because:
//! - Config says: `pipewire`
//! - AUR installs: `pipewire-full`
//! - `pacman -Q` shows: `pipewire` (the provided name)
//!
//! # Solution
//!
//! Track multiple identities:
//! - **config_name**: What user wrote in config (e.g., "hyprland")
//! - **provides_name**: What `pacman -Q` shows (e.g., "hyprland")
//! - **aur_package_name**: Actual AUR package (e.g., "hyprland-git")
//!
//! Then use provides_name as the primary identity for matching.

use crate::core::types::{Backend, PackageId};
use std::fmt;

/// Identity of a package across different contexts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageIdentity {
    /// Backend (aur, flatpak, etc.)
    pub backend: Backend,

    /// Name from config file (what user wrote)
    pub config_name: String,

    /// Name that `pacman -Q` shows (provided name)
    /// This is the primary identity for matching
    pub provides_name: String,

    /// Actual AUR/Flatpak package name
    /// For AUR: this might differ from provides_name
    /// For Flatpak: usually the same as provides_name
    pub aur_package_name: Option<String>,
}

impl PackageIdentity {
    /// Create identity from config package name
    ///
    /// When user writes package in config, we start with:
    /// - config_name = what user wrote
    /// - provides_name = same as config (we'll discover the actual later)
    /// - aur_package_name = None (unknown until we query system)
    pub fn from_config(config_name: String, backend: Backend) -> Self {
        Self {
            backend,
            config_name: config_name.clone(),
            provides_name: config_name,
            aur_package_name: None,
        }
    }

    /// Create identity from installed package
    ///
    /// When we discover what's actually installed:
    /// - provides_name: what `pacman -Q` shows
    /// - aur_package_name: the actual package name
    pub fn from_installed(
        provides_name: String,
        aur_package_name: Option<String>,
        backend: Backend,
    ) -> Self {
        Self {
            backend,
            config_name: provides_name.clone(), // Assume matches initially
            provides_name,
            aur_package_name,
        }
    }

    /// Check if this identity refers to the same package as another
    ///
    /// Same package = same provides_name (ignores variants)
    /// Example:
    /// - hyprland vs hyprland-git → SAME package (different variants)
    /// - hyprland vs wayland → DIFFERENT packages
    pub fn is_same_package(&self, other: &Self) -> bool {
        self.provides_name == other.provides_name && self.backend == other.backend
    }

    /// Check if this is a variant transition (same package, different variant)
    ///
    /// Variant transition = same provides_name but different aur_package_name
    /// Example: hyprland → hyprland-git
    pub fn is_variant_transition(&self, other: &Self) -> bool {
        if self.provides_name != other.provides_name {
            return false;
        }

        match (&self.aur_package_name, &other.aur_package_name) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        }
    }

    /// Get the primary key for state storage
    ///
    /// Uses provides_name as the key since that's what pacman shows
    pub fn state_key(&self) -> String {
        format!("{}:{}", self.backend, self.provides_name)
    }

    /// Create PackageId for core operations
    pub fn to_package_id(&self) -> PackageId {
        PackageId {
            name: self.provides_name.clone(),
            backend: self.backend.clone(),
        }
    }

    /// Get display name for user messages
    pub fn display_name(&self) -> String {
        if let Some(aur_pkg) = &self.aur_package_name {
            if aur_pkg != &self.provides_name {
                format!("{} (installed as {})", self.config_name, aur_pkg)
            } else {
                self.config_name.clone()
            }
        } else {
            self.config_name.clone()
        }
    }
}

impl fmt::Display for PackageIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.aur_package_name {
            Some(aur) if aur != &self.provides_name => {
                write!(f, "{} [{}]", self.config_name, aur)
            }
            _ => write!(f, "{}", self.config_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_from_config() {
        let identity = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));

        assert_eq!(identity.config_name, "hyprland");
        assert_eq!(identity.provides_name, "hyprland");
        assert_eq!(identity.aur_package_name, None);
    }

    #[test]
    fn test_identity_from_installed() {
        let identity = PackageIdentity::from_installed(
            "hyprland".to_string(),
            Some("hyprland-git".to_string()),
            Backend::from("aur"),
        );

        assert_eq!(identity.config_name, "hyprland");
        assert_eq!(identity.provides_name, "hyprland");
        assert_eq!(identity.aur_package_name, Some("hyprland-git".to_string()));
    }

    #[test]
    fn test_is_same_package() {
        let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
        let id2 = PackageIdentity::from_installed(
            "hyprland".to_string(),
            Some("hyprland-git".to_string()),
            Backend::from("aur"),
        );

        assert!(id1.is_same_package(&id2));
    }

    #[test]
    fn test_is_variant_transition() {
        let id1 = PackageIdentity::from_installed(
            "hyprland".to_string(),
            Some("hyprland".to_string()),
            Backend::from("aur"),
        );
        let id2 = PackageIdentity::from_installed(
            "hyprland".to_string(),
            Some("hyprland-git".to_string()),
            Backend::from("aur"),
        );

        assert!(id1.is_variant_transition(&id2));
    }

    #[test]
    fn test_different_packages() {
        let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
        let id2 = PackageIdentity::from_config("wayland".to_string(), Backend::from("aur"));

        assert!(!id1.is_same_package(&id2));
        assert!(!id1.is_variant_transition(&id2));
    }

    #[test]
    fn test_display_name() {
        let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
        assert_eq!(id1.display_name(), "hyprland");

        let id2 = PackageIdentity::from_installed(
            "hyprland".to_string(),
            Some("hyprland-git".to_string()),
            Backend::from("aur"),
        );
        assert_eq!(id2.display_name(), "hyprland (installed as hyprland-git)");
    }
}
