//! # Package Manager Registry
//!
//! This module manages the registration and creation of package manager instances.
//!
//! ## Backend Implementation Patterns
//!
//! Declarch supports **two different backend implementation patterns**:
//!
//! ### 1. Custom Rust Implementations (src/packages/*.rs)
//!
//! These backends have complex logic that cannot be expressed in simple configuration.
//!
//! - **AUR** (`packages/aur.rs`): Paru/Yay detection, special AUR handling
//! - **Flatpak** (`packages/flatpak.rs`): Remote management, special installation patterns
//! - **Soar** (`packages/soar.rs`): Auto-installation, static binary management
//!
//! ### 2. Generic Config-Driven Implementations (src/backends/registry.rs)
//!
//! These backends follow simple command patterns and are configuration-driven.
//!
//! - **npm, yarn, pnpm, bun**: Node.js package managers (install/remove/list)
//! - **pip**: Python package manager
//! - **cargo**: Rust package manager
//! - **brew**: Homebrew package manager
//!
//! ## When to Use Which Pattern?
//!
//! **Use Custom Implementation** when:
//! - Backend requires complex state management
//! - Special detection/initialization logic needed
//! - Non-standard command patterns
//! - Backend-specific features (e.g., AUR helpers)
//!
//! **Use Generic Implementation** when:
//! - Backend follows standard package manager pattern
//! - Simple install/remove/list commands
//! - No special initialization required
//! - Can be configured declaratively
//!
//! ## Adding a New Backend
//!
//! For a **custom backend**:
//! 1. Create `src/packages/<backend>.rs` with Manager struct
//! 2. Implement `PackageManager` trait
//! 3. Add `Backend::<Name>` variant to `core/types.rs`
//! 4. Register in `BackendRegistry::register_defaults()`
//!
//! For a **generic backend**:
//! 1. Add configuration to `src/backends/registry.rs::get_builtin_backends()`
//! 2. Add `Backend::<Name>` variant to `core/types.rs`
//! 3. Register in `BackendRegistry::register_defaults()` using `GenericManager`

use crate::backends::{GenericManager, get_builtin_backends};
use crate::config::types::GlobalConfig;
use crate::core::types::Backend;
use crate::packages::PackageManager;
use crate::utils::distro::DistroType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Factory function for creating package manager instances
pub type BackendFactory =
    Box<dyn Fn(&GlobalConfig, bool) -> Result<Box<dyn PackageManager>, String> + Send + Sync>;

/// Backend registry for managing available package managers
///
/// This registry allows dynamic backend registration and initialization,
/// making it easy to add new package managers (nala, nix, etc.) without
/// modifying core logic.
///
/// # Adding a New Backend
///
/// To add a new backend (e.g., Nix):
/// 1. Create NixManager implementing PackageManager
/// 2. Add Backend::Nix to core::types::Backend enum
/// 3. Register in BackendRegistry::register_defaults():
///
/// ```no_run
/// # use declarch::packages::registry::{BackendRegistry, BackendFactory};
/// # use declarch::core::types::Backend;
/// # use declarch::config::types::GlobalConfig;
/// # let mut registry = BackendRegistry::new();
/// // This is example code showing the pattern:
/// // self.register(Backend::Nix, |config, noconfirm| {
/// //     Ok(Box::new(NixManager::new(noconfirm)))
/// // });
/// ```
pub struct BackendRegistry {
    factories: HashMap<Backend, BackendFactory>,
}

impl BackendRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a backend with its factory function
    pub fn register<F>(&mut self, backend: Backend, factory: F)
    where
        F: Fn(&GlobalConfig, bool) -> Result<Box<dyn PackageManager>, String>
            + Send
            + Sync
            + 'static,
    {
        self.factories.insert(backend, Box::new(factory));
    }

    /// Create a package manager instance for the given backend
    pub fn create_manager(
        &self,
        backend: &Backend,
        config: &GlobalConfig,
        noconfirm: bool,
    ) -> Result<Box<dyn PackageManager>, String> {
        // First try to get factory from registry
        if let Some(factory) = self.factories.get(backend) {
            return factory(config, noconfirm);
        }

        // If not found, check if it's a Custom backend and load dynamically
        match backend {
            Backend::Custom(backend_name) => {
                // Load user-defined backends and create GenericManager
                let all_backends = crate::backends::load_all_backends()
                    .map_err(|e| format!("Failed to load backends: {}", e))?;

                let backend_config = all_backends.get(backend_name).ok_or_else(|| {
                    format!(
                        "Custom backend '{}' not found in backends.kdl",
                        backend_name
                    )
                })?;

                Ok(Box::new(GenericManager::from_config(
                    backend_config.clone(),
                    backend.clone(),
                    noconfirm,
                )))
            }
            _ => Err(format!("No factory registered for backend: {}", backend)),
        }
    }

    /// Get all registered backends
    pub fn registered_backends(&self) -> Vec<Backend> {
        self.factories.keys().cloned().collect()
    }

    /// Check if a backend is registered
    pub fn has_backend(&self, backend: &Backend) -> bool {
        self.factories.contains_key(backend)
    }

    /// Register all default backends
    ///
    /// This method registers all built-in backends. New backends should
    /// be added here following the existing pattern.
    pub fn register_defaults(&mut self) {
        // AUR Backend (Arch Linux) - Uses Rust implementation
        self.register(Backend::Aur, |config, noconfirm| {
            Ok(Box::new(crate::packages::aur::AurManager::new(
                config.aur_helper.to_string(),
                noconfirm,
            )))
        });

        // Flatpak Backend (Cross-distro) - Uses Rust implementation
        self.register(Backend::Flatpak, |_config, noconfirm| {
            Ok(Box::new(crate::packages::flatpak::FlatpakManager::new(
                noconfirm,
            )))
        });

        // Soar Backend (Cross-distro) - Uses Rust implementation
        self.register(Backend::Soar, |_config, noconfirm| {
            Ok(Box::new(crate::packages::soar::SoarManager::new(noconfirm)))
        });

        // === New Generic Backends ===
        // Get built-in backend configurations
        let builtin_backends = get_builtin_backends();

        // Register npm backend
        if let Some(npm_config) = builtin_backends.get("npm") {
            let config = npm_config.clone();
            self.register(Backend::Npm, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Npm,
                    noconfirm,
                )))
            });
        }

        // Register yarn backend
        if let Some(yarn_config) = builtin_backends.get("yarn") {
            let config = yarn_config.clone();
            self.register(Backend::Yarn, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Yarn,
                    noconfirm,
                )))
            });
        }

        // Register pnpm backend
        if let Some(pnpm_config) = builtin_backends.get("pnpm") {
            let config = pnpm_config.clone();
            self.register(Backend::Pnpm, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Pnpm,
                    noconfirm,
                )))
            });
        }

        // Register bun backend
        if let Some(bun_config) = builtin_backends.get("bun") {
            let config = bun_config.clone();
            self.register(Backend::Bun, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Bun,
                    noconfirm,
                )))
            });
        }

        // Register pip backend
        if let Some(pip_config) = builtin_backends.get("pip") {
            let config = pip_config.clone();
            self.register(Backend::Pip, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Pip,
                    noconfirm,
                )))
            });
        }

        // Register cargo backend
        if let Some(cargo_config) = builtin_backends.get("cargo") {
            let config = cargo_config.clone();
            self.register(Backend::Cargo, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Cargo,
                    noconfirm,
                )))
            });
        }

        // Register brew backend
        if let Some(brew_config) = builtin_backends.get("brew") {
            let config = brew_config.clone();
            self.register(Backend::Brew, move |_global_config, noconfirm| {
                Ok(Box::new(GenericManager::from_config(
                    config.clone(),
                    Backend::Brew,
                    noconfirm,
                )))
            });
        }
    }

    /// Get available backends for the current distro
    ///
    /// On Arch: Returns AUR + Soar + Flatpak
    /// On non-Arch: Returns Soar + Flatpak (AUR is skipped)
    pub fn available_backends(&self, distro: &DistroType) -> Vec<Backend> {
        let mut backends = Vec::new();

        for backend in self.registered_backends() {
            match backend {
                Backend::Aur => {
                    // Only AUR on Arch-based systems
                    if distro.supports_aur() {
                        backends.push(backend);
                    }
                }
                Backend::Soar
                | Backend::Flatpak
                | Backend::Npm
                | Backend::Yarn
                | Backend::Pnpm
                | Backend::Bun
                | Backend::Pip
                | Backend::Cargo
                | Backend::Brew
                | Backend::Custom(_) => {
                    // These work on all distros
                    backends.push(backend);
                }
            }
        }

        backends
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }
}

// Global lazy-initialized registry
use std::sync::OnceLock;

static REGISTRY: OnceLock<Arc<Mutex<BackendRegistry>>> = OnceLock::new();

/// Get the global backend registry
pub fn get_registry() -> Arc<Mutex<BackendRegistry>> {
    REGISTRY
        .get_or_init(|| {
            let mut registry = BackendRegistry::new();
            registry.register_defaults();
            Arc::new(Mutex::new(registry))
        })
        .clone()
}

/// Create a package manager using the global registry
pub fn create_manager(
    backend: &Backend,
    config: &GlobalConfig,
    noconfirm: bool,
) -> Result<Box<dyn PackageManager>, String> {
    let registry = get_registry();
    let registry_guard = registry
        .lock()
        .map_err(|e| format!("Registry lock failed: {}", e))?;

    registry_guard.create_manager(backend, config, noconfirm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_registration() {
        let mut registry = BackendRegistry::new();
        registry.register_defaults();

        // Check that default backends are registered
        assert!(registry.has_backend(&Backend::Aur));
        assert!(registry.has_backend(&Backend::Flatpak));
        assert!(registry.has_backend(&Backend::Soar));
    }

    #[test]
    fn test_create_manager() {
        let registry = BackendRegistry::default();
        let config = GlobalConfig::default();

        // Test creating managers
        for backend in registry.registered_backends() {
            let result = registry.create_manager(&backend, &config, false);
            assert!(result.is_ok(), "Failed to create manager for {:?}", backend);
        }
    }

    #[test]
    fn test_available_backends_arch() {
        let registry = BackendRegistry::default();
        let distro = DistroType::Arch;

        let backends = registry.available_backends(&distro);

        // Arch should have all backends
        assert!(backends.contains(&Backend::Aur));
        assert!(backends.contains(&Backend::Flatpak));
        assert!(backends.contains(&Backend::Soar));
    }

    #[test]
    fn test_available_backends_debian() {
        let registry = BackendRegistry::default();
        let distro = DistroType::Debian;

        let backends = registry.available_backends(&distro);

        // Debian should not have AUR
        assert!(!backends.contains(&Backend::Aur));
        assert!(backends.contains(&Backend::Flatpak));
        assert!(backends.contains(&Backend::Soar));
    }

    #[test]
    fn test_custom_backend_registration() {
        let mut registry = BackendRegistry::new();

        // Register a mock backend
        registry.register(Backend::Aur, |_config, _noconfirm| {
            Ok(Box::new(crate::packages::aur::AurManager::new(
                "paru".to_string(),
                false,
            )))
        });

        assert!(registry.has_backend(&Backend::Aur));

        let config = GlobalConfig::default();
        let manager = registry.create_manager(&Backend::Aur, &config, false);
        assert!(manager.is_ok());
    }
}
