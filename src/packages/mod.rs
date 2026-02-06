//! # Package Manager Module
//!
//! This module provides the core infrastructure for package management.
//! All backends are now handled through the generic `GenericManager`.
//!
//! ## Architecture (v0.6+)
//!
//! In v0.6, all package managers are config-driven through `GenericManager`.
//! There are no custom Rust implementations - all backend logic is defined
//! in KDL configuration files at `~/.config/declarch/backends/*.kdl`.
//!
//! ## PackageManager Trait
//!
//! The `PackageManager` trait defines the interface that all backends implement:
//! - `install()` - Install packages
//! - `remove()` - Remove packages
//! - `list_installed()` - List installed packages
//! - `is_available()` - Check if package manager is available
//!
//! ## BackendRegistry
//!
//! The `BackendRegistry` provides a factory for creating package manager
//! instances from backend configuration files.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use declarch::packages::create_manager;
//! use declarch::core::types::Backend;
//! use declarch::config::types::GlobalConfig;
//!
//! let config = GlobalConfig::default();
//! let backend = Backend::from("paru");
//! let manager = create_manager(&backend, &config, false)?;
//! manager.install(&vec!["hyprland".to_string()])?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod registry;
pub mod traits;

pub use registry::{BackendRegistry, create_manager, get_registry};
pub use traits::PackageManager;
