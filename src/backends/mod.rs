//! # Backend Configuration and Generic Manager
//!
//! This module provides the infrastructure for **generic, config-driven package managers**.
//!
//! ## Overview
//!
//! Simple package managers (npm, pip, cargo, brew, etc.) are supported through a
//! unified `GenericManager` that reads declarative configuration. This avoids writing
//! custom Rust code for each package manager.
//!
//! ## Module Structure
//!
//! ### [`config`] - Backend Configuration
//!
//! Defines the `BackendConfig` struct that describes how to interact with a package manager:
//! - Command templates (install, remove, list)
//! - Output format (JSON, TSV, whitespace-separated)
//! - Binary detection (single binary or multiple options)
//! - Special flags (noconfirm, sudo requirements)
//!
//! ### [`generic`] - Generic Package Manager
//!
//! `GenericManager` implements the `PackageManager` trait for any package manager
//! that can be described via `BackendConfig`. It handles:
//! - Parsing command output (via output parsers)
//! - Executing install/remove/list commands
//! - Managing environment variables
//! - Multi-binary fallback (e.g., pip3 → pip)
//!
//! ### [`parsers`] - Output Parsers
//!
//! Parser implementations for different command output formats:
//! - **JSON parser**: Parse structured JSON output
//! - **Regex parser**: Extract data via regular expressions
//! - **TSV parser**: Tab-separated values
//! - **Whitespace parser**: Split by whitespace
//!
//! ### [`registry`] - Built-in Backend Configurations
//!
//! Contains pre-configured `BackendConfig` for all supported generic backends:
//! - Node.js: npm, yarn, pnpm, bun
//! - Python: pip
//! - Rust: cargo
//! - macOS/Linux: brew
//!
//! ### [`user_parser`] - User-Defined Backends
//!
//! Loads custom backend definitions from `~/.config/declarch/backends.kdl`,
//! allowing users to add support for additional package managers without
//! modifying the codebase.
//!
//! ## When to Use Generic vs Custom Implementations
//!
//! **Use GenericManager** when:
//! - Package manager has standard install/remove/list commands
//! - Output can be parsed with existing parsers
//! - No complex state management required
//! - No special initialization needed
//!
//! **Use Custom Implementation** (`packages/` module) when:
//! - Package manager requires complex logic (AUR helpers, Flatpak remotes)
//! - Special detection or initialization needed
//! - Non-standard command patterns
//! - Backend-specific features required
//!
//! ## Example: Adding a New Generic Backend
//!
//! ```rust,no_run
//! use declarch::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
//!
//! let config = BackendConfig {
//!     name: "mybackend".to_string(),
//!     binary: BinarySpecifier::Single("mybackend".to_string()),
//!     list_cmd: "mybackend list --json".to_string(),
//!     install_cmd: "mybackend install {packages}".to_string(),
//!     remove_cmd: "mybackend uninstall {packages}".to_string(),
//!     query_cmd: None,
//!     list_format: OutputFormat::Json,
//!     list_name_col: None,
//!     list_version_col: None,
//!     list_json_path: Some("packages".to_string()),
//!     list_name_key: Some("name".to_string()),
//!     list_version_key: Some("version".to_string()),
//!     list_regex: None,
//!     list_regex_name_group: None,
//!     list_regex_version_group: None,
//!     noconfirm_flag: Some("--yes".to_string()),
//!     needs_sudo: false,
//!     preinstall_env: None,
//!     use_rust_fallback: false,
//!     search_cmd: None,
//!     search_format: None,
//!     search_json_path: None,
//!     search_name_key: None,
//!     search_version_key: None,
//!     search_desc_key: None,
//!     search_name_col: None,
//!     search_desc_col: None,
//!     search_regex: None,
//!     search_regex_name_group: None,
//!     search_regex_desc_group: None,
//! };
//! ```
//!
//! ## User-Defined Backends
//!
//! Users can define custom backends in `~/.config/declarch/backends.kdl`:
//!
//! ```kdl
//! backend "mybackend" {
//!     binary "mybackend"
//!     list_cmd "mybackend list --json"
//!     install_cmd "mybackend install {packages}"
//!     remove_cmd "mybackend uninstall {packages}"
//!     format json
//!     json_path "packages"
//!     name_key "name"
//!     version_key "version"
//! }
//! ```
//!
//! User backends override built-in configurations, allowing customization.

pub mod config;
pub mod generic;
pub mod parsers;
pub mod registry;
pub mod user_parser;

#[cfg(test)]
mod integration_tests;

pub use config::{BackendConfig, BinarySpecifier, OutputFormat};
pub use generic::GenericManager;

// Re-export commonly used items
pub use registry::{get_builtin_backends, load_all_backends};
