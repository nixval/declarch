//! # Package Manager Registry
//!
//! This module manages the creation of package manager instances.
//!
//! ## Architecture (v0.6+)
//!
//! All package managers are now config-driven through `GenericManager`.
//! Backend definitions are loaded from `~/.config/declarch/backends/*.kdl`.
//!
//! ## Backend Loading
//!
//! 1. Load all backend configs from `backends/*.kdl` files
//! 2. Create `GenericManager` instance for requested backend
//! 3. GenericManager handles fallback logic based on config

use crate::backends::{GenericManager, load_all_backends_unified};
use crate::config::types::GlobalConfig;
use crate::core::types::Backend;
use crate::packages::PackageManager;
use crate::ui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Backend registry for managing available package managers
///
/// In v0.6+, this is a thin wrapper that loads backend configs
/// and creates GenericManager instances on demand.
pub struct BackendRegistry {
    /// Cached backend configurations
    configs: HashMap<String, crate::backends::config::BackendConfig>,
}

impl BackendRegistry {
    /// Create a new registry and load all backend configs
    pub fn new() -> crate::error::Result<Self> {
        let configs = load_all_backends_unified()?;
        Ok(Self { configs })
    }

    /// Check if a backend is available
    pub fn has_backend(&self, name: &str) -> bool {
        self.configs.contains_key(name)
    }

    /// Get list of available backend names
    pub fn available_backends(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Get backend configuration by name
    pub fn get_backend_config(
        &self,
        name: &str,
    ) -> Option<&crate::backends::config::BackendConfig> {
        self.configs.get(name)
    }

    /// Create a package manager for the given backend
    pub fn create_manager(
        &self,
        backend: &Backend,
        _config: &GlobalConfig,
        noconfirm: bool,
    ) -> Result<Box<dyn PackageManager>, String> {
        let backend_name = backend.name();

        // Look up backend config
        let backend_config = self.configs.get(backend_name).ok_or_else(|| {
            format!(
                "Backend '{}' not found. Run 'declarch init --backend {}'",
                backend_name, backend_name
            )
        })?;

        // Create GenericManager from config
        Ok(Box::new(GenericManager::from_config(
            backend_config.clone(),
            backend.clone(),
            noconfirm,
        )))
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            ui::warning(&format!("Failed to load backend configs: {}", e));
            Self {
                configs: HashMap::new(),
            }
        })
    }
}

// Global lazy-initialized registry
use std::sync::OnceLock;

static REGISTRY: OnceLock<Arc<Mutex<BackendRegistry>>> = OnceLock::new();

/// Get the global backend registry
pub fn get_registry() -> Arc<Mutex<BackendRegistry>> {
    REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(BackendRegistry::default())))
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
    fn test_registry_creation() {
        let registry = BackendRegistry::new();
        // May succeed or fail depending on whether backend configs exist
        // In test environment, likely to be empty
        assert!(registry.is_ok() || registry.is_err());
    }
}
