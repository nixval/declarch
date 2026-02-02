//! # Package Manager Implementations
//!
//! This module contains custom Rust implementations for package managers that
//! require complex logic beyond what can be expressed in simple configuration.
//!
//! ## Architecture
//!
//! Declarch supports two backend patterns:
//!
//! ### Custom Implementations (This Module)
//!
//! - **AUR** (`aur.rs`): Arch User Repository with helper detection
//! - **Flatpak** (`flatpak.rs`): Universal apps with remote management
//! - **Soar** (`soar.rs`): Static binary registry with auto-installation
//!
//! These backends require custom Rust code because they have:
//! - Complex state management
//! - Special initialization logic
//! - Non-standard command patterns
//! - Backend-specific features
//!
//! ### Generic Implementations (backends/ module)
//!
//! Simple package managers (npm, pip, cargo, brew, etc.) use the config-driven
//! `GenericManager` pattern defined in the `backends` module.
//!
//! ## PackageManager Trait
//!
//! All package managers implement the `PackageManager` trait, which defines:
//! - `install()` - Install packages
//! - `remove()` - Remove packages
//! - `list_installed()` - List installed packages
//! - `is_available()` - Check if package manager is available
//!
//! ## BackendRegistry
//!
//! The `BackendRegistry` provides a factory pattern for creating package manager
//! instances. It registers both custom and generic backends, allowing unified
//! access through a common interface.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use declarch::packages::create_manager;
//! use declarch::core::types::Backend;
//! use declarch::config::types::GlobalConfig;
//!
//! let config = GlobalConfig::default();
//! let manager = create_manager(&Backend::Aur, &config, false)?;
//! manager.install(&vec!["hyprland".to_string()])?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod aur;
pub mod registry;
pub mod soar;
pub mod traits;

pub use registry::{create_manager, get_registry, BackendRegistry};
pub use traits::PackageManager;
