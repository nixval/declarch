//! Core traits for dependency injection and testability
//!
//! This module defines the core abstractions that allow declarch to be
//! testable and flexible. By depending on traits rather than concrete
//! implementations, we can:
//! - Mock dependencies for unit tests
//! - Swap implementations (e.g., different state backends)
//! - Avoid global mutable state

use crate::backends::config::BackendConfig;
use crate::config::loader::MergedConfig;
use crate::error::Result;
use crate::state::types::State;
use std::collections::HashMap;

/// Trait for loading configuration files
///
/// This abstraction allows us to:
/// - Load from different sources (filesystem, remote, mock)
/// - Cache configurations
/// - Implement config reloading
pub trait ConfigLoader: Send + Sync {
    /// Load the root configuration from a path
    fn load_root(&self, path: &std::path::Path) -> Result<MergedConfig>;

    /// Check if a config file exists
    fn exists(&self, path: &std::path::Path) -> bool;
}

/// Trait for state persistence
///
/// This abstraction allows us to:
/// - Store state in different backends (filesystem, database, etc.)
/// - Mock state for testing
/// - Implement state migration
pub trait StateStore: Send + Sync {
    /// Load state from storage
    fn load(&self) -> Result<State>;

    /// Save state to storage
    fn save(&self, state: &State) -> Result<()>;

    /// Initialize empty state if none exists
    fn init(&self) -> Result<State>;
}

/// Trait for backend registry
///
/// This abstraction allows us to:
/// - Load backends from different sources
/// - Cache backend configurations
/// - Mock backends for testing
pub trait BackendRegistry: Send + Sync {
    /// Load all available backends
    fn load_all(&self) -> Result<HashMap<String, BackendConfig>>;

    /// Get a specific backend by name
    fn get(&self, name: &str) -> Result<Option<BackendConfig>>;

    /// Check if a backend exists
    fn has(&self, name: &str) -> bool;
}

/// Application context holding all dependencies
///
/// This struct is passed to commands and provides access to all
/// external dependencies. This enables dependency injection and
/// makes testing much easier.
pub struct AppContext {
    /// Configuration loader
    pub config_loader: Box<dyn ConfigLoader>,
    /// State store
    pub state_store: Box<dyn StateStore>,
    /// Backend registry
    pub backend_registry: Box<dyn BackendRegistry>,
}

impl AppContext {
    /// Create a new context with the given dependencies
    pub fn new(
        config_loader: Box<dyn ConfigLoader>,
        state_store: Box<dyn StateStore>,
        backend_registry: Box<dyn BackendRegistry>,
    ) -> Self {
        Self {
            config_loader,
            state_store,
            backend_registry,
        }
    }

    /// Create a new context using filesystem-based implementations
    pub fn new_filesystem() -> Result<Self> {
        Ok(Self {
            config_loader: Box::new(crate::config::loader::FilesystemConfigLoader),
            state_store: Box::new(crate::state::io::FilesystemStateStore),
            backend_registry: Box::new(crate::backends::registry::FilesystemBackendRegistry),
        })
    }
}

impl Default for AppContext {
    /// Create a default context using filesystem-based implementations
    fn default() -> Self {
        Self {
            config_loader: Box::new(crate::config::loader::FilesystemConfigLoader),
            state_store: Box::new(crate::state::io::FilesystemStateStore),
            backend_registry: Box::new(crate::backends::registry::FilesystemBackendRegistry),
        }
    }
}

// Default implementations are in their respective modules
// This module only defines the traits
