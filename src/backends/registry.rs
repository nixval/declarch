//! # Built-in Backend Configurations
//!
//! This module defines built-in backend configurations for package managers that
//! use the generic config-driven pattern (via `GenericManager`).
//!
//! ## Generic Backend Pattern
//!
//! Generic backends are package managers that follow standard patterns:
//! - Simple install/remove/list commands
//! - No complex state management
//! - No special initialization logic
//! - Can be fully configured via `BackendConfig`
//!
//! ## Supported Generic Backends
//!
//! ### Node.js Ecosystem
//! - **npm**: Node package manager
//! - **yarn**: Package manager for Node (uses npm for listing)
//! - **pnpm**: Fast, disk space efficient package manager
//! - **bun**: Ultra-fast Node.js runtime and package manager
//!
//! ### Python
//! - **pip**: Python package manager (tries pip3 first, then pip)
//!
//! ### Rust
//! - **cargo**: Rust package manager
//!
//! ### Cross-Platform
//! - **brew**: Homebrew package manager (Linux/macOS)
//!
//! ## Backend Configuration Structure
//!
//! Each backend defines:
//! - `binary`: How to invoke the package manager
//! - `list_cmd`: Command to list installed packages
//! - `install_cmd`: Command to install packages
//! - `remove_cmd`: Command to remove packages
//! - `list_format`: Output format (Json, SplitWhitespace, etc.)
//! - `needs_sudo`: Whether sudo is required
//! - `noconfirm_flag`: Auto-confirmation flag
//!
//! ## Adding a New Generic Backend
//!
//! To add support for a new package manager:
//!
//! ```no_run
//! # use declarch::backends::config::{BackendConfig, BinarySpecifier};
//! # use declarch::backends::config::OutputFormat;
//! # let mut backends = std::collections::HashMap::new();
//! // Add to get_builtin_backends():
//! backends.insert(
//!     "newpm".to_string(),
//!     BackendConfig {
//!         name: "newpm".to_string(),
//!         binary: BinarySpecifier::Single("newpm".to_string()),
//!         list_cmd: "newpm list --json 2>/dev/null".to_string(),
//!         install_cmd: "newpm install {packages}".to_string(),
//!         remove_cmd: "newpm uninstall {packages}".to_string(),
//!         query_cmd: None,
//!         list_format: OutputFormat::Json,
//!         list_name_col: None,
//!         list_version_col: None,
//!         list_json_path: Some("dependencies".to_string()),
//!         list_name_key: Some("name".to_string()),
//!         list_version_key: Some("version".to_string()),
//!         list_regex: None,
//!         list_regex_name_group: None,
//!         list_regex_version_group: None,
//!         noconfirm_flag: Some("--yes".to_string()),
//!         needs_sudo: false,
//!         preinstall_env: None,
//!         use_rust_fallback: false,
//!     },
//! );
//! ```
//!
//! Then register it in `packages/registry.rs::register_defaults()`.

use crate::backends::config::{BackendConfig, BinarySpecifier};
use crate::backends::user_parser;
use crate::utils::paths;
use std::collections::HashMap;

/// Get all backend configurations (built-in + user-defined)
///
/// User-defined backends from ~/.config/declarch/backends.kdl
/// can override built-in backends.
pub fn load_all_backends() -> crate::error::Result<HashMap<String, BackendConfig>> {
    let mut all_backends = get_builtin_backends();

    // Load user-defined backends
    let backends_path = paths::backend_config()?;

    if backends_path.exists() {
        let user_backends = user_parser::load_user_backends(&backends_path)?;

        // User backends override built-ins
        for config in user_backends {
            all_backends.insert(config.name.clone(), config);
        }
    }

    Ok(all_backends)
}

/// Get built-in backend configurations
pub fn get_builtin_backends() -> HashMap<String, BackendConfig> {
    let mut backends = HashMap::new();

    // === NPM Backend ===
    backends.insert(
        "npm".to_string(),
        BackendConfig {
            name: "npm".to_string(),
            binary: BinarySpecifier::Single("npm".to_string()),
            list_cmd: "npm list -g --depth=0 --json 2>/dev/null".to_string(),
            install_cmd: "npm install -g {packages}".to_string(),
            remove_cmd: "npm uninstall -g {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::Json,
            list_name_col: None,
            list_version_col: None,
            list_json_path: Some("dependencies".to_string()),
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None, // npm doesn't have a noconfirm flag
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    // === Yarn Backend ===
    backends.insert(
        "yarn".to_string(),
        BackendConfig {
            name: "yarn".to_string(),
            binary: BinarySpecifier::Single("yarn".to_string()),
            list_cmd: "npm list -g --depth=0 --json 2>/dev/null".to_string(), // Use npm list (yarn global is messy)
            install_cmd: "yarn global add {packages}".to_string(),
            remove_cmd: "yarn global remove {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::Json,
            list_name_col: None,
            list_version_col: None,
            list_json_path: Some("dependencies".to_string()),
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None,
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    // === PNPM Backend ===
    backends.insert(
        "pnpm".to_string(),
        BackendConfig {
            name: "pnpm".to_string(),
            binary: BinarySpecifier::Single("pnpm".to_string()),
            list_cmd: "pnpm list -g --depth=0 --json 2>/dev/null".to_string(),
            install_cmd: "pnpm add -g {packages}".to_string(),
            remove_cmd: "pnpm remove -g {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::Json,
            list_name_col: None,
            list_version_col: None,
            list_json_path: Some("".to_string()), // Root array
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None,
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    // === BUN Backend ===
    backends.insert(
        "bun".to_string(),
        BackendConfig {
            name: "bun".to_string(),
            binary: BinarySpecifier::Single("bun".to_string()),
            list_cmd: "bun pm ls -g --json".to_string(),
            install_cmd: "bun install -g {packages}".to_string(),
            remove_cmd: "bun remove -g {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::Json,
            list_name_col: None,
            list_version_col: None,
            list_json_path: Some("".to_string()), // Root array
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None,
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    // === PIP Backend ===
    backends.insert(
        "pip".to_string(),
        BackendConfig {
            name: "pip".to_string(),
            binary: BinarySpecifier::Multiple(vec!["pip3".to_string(), "pip".to_string()]),
            list_cmd: "pip list --format=json 2>/dev/null".to_string(),
            install_cmd: "pip install --break-system-packages {packages}".to_string(),
            remove_cmd: "pip uninstall -y {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::Json,
            list_name_col: None,
            list_version_col: None,
            list_json_path: Some("".to_string()), // Root array
            list_name_key: Some("name".to_string()),
            list_version_key: Some("version".to_string()),
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: Some("-y".to_string()),
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    // === Cargo Backend ===
    backends.insert(
        "cargo".to_string(),
        BackendConfig {
            name: "cargo".to_string(),
            binary: BinarySpecifier::Single("cargo".to_string()),
            list_cmd: "cargo install --list 2>/dev/null || echo ''".to_string(),
            install_cmd: "cargo install {packages}".to_string(),
            remove_cmd: "cargo uninstall {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::SplitWhitespace,
            list_name_col: Some(0),    // First column: package name
            list_version_col: Some(1), // Second column: version
            list_json_path: None,
            list_name_key: None,
            list_version_key: None,
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: None,
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false, // Now uses generic parser
        },
    );

    // === Homebrew Backend ===
    backends.insert(
        "brew".to_string(),
        BackendConfig {
            name: "brew".to_string(),
            binary: BinarySpecifier::Single("brew".to_string()),
            list_cmd: "brew list --versions".to_string(),
            install_cmd: "brew install {packages}".to_string(),
            remove_cmd: "brew uninstall {packages}".to_string(),
            query_cmd: None,
            list_format: crate::backends::config::OutputFormat::SplitWhitespace,
            list_name_col: Some(0),
            list_version_col: None, // Multiple versions possible
            list_json_path: None,
            list_name_key: None,
            list_version_key: None,
            list_regex: None,
            list_regex_name_group: None,
            list_regex_version_group: None,
            noconfirm_flag: Some("--force".to_string()),
            needs_sudo: false,
            preinstall_env: None,
            use_rust_fallback: false,
        },
    );

    backends
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_backends() {
        let backends = get_builtin_backends();

        assert!(backends.contains_key("npm"));
        assert!(backends.contains_key("pip"));
        assert!(backends.contains_key("cargo"));
        assert!(backends.contains_key("brew"));
    }

    #[test]
    fn test_load_all_backends() {
        let backends = load_all_backends().unwrap();

        // Should contain all built-in backends
        assert!(backends.contains_key("npm"));
        assert!(backends.contains_key("pip"));
        assert!(backends.contains_key("cargo"));
        assert!(backends.contains_key("brew"));
    }

    #[test]
    fn test_npm_config() {
        let backends = get_builtin_backends();
        let npm = &backends["npm"];

        assert_eq!(npm.name, "npm");
        assert!(!npm.needs_sudo);
        assert_eq!(npm.list_format, crate::backends::config::OutputFormat::Json);
    }

    #[test]
    fn test_pip_config() {
        let backends = get_builtin_backends();
        let pip = &backends["pip"];

        assert_eq!(pip.name, "pip");
        assert!(!pip.needs_sudo);
        assert_eq!(pip.list_format, crate::backends::config::OutputFormat::Json);
    }
}
