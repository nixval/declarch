//! Input sanitization utilities for security
//!
//! This module provides validation and sanitization functions
//! to prevent command injection and other security issues.

use crate::error::{DeclarchError, Result};
use regex::Regex;
use std::sync::LazyLock;

/// Safe characters for package names across different package managers
/// Allows: alphanumeric, dash, underscore, dot, plus, at sign, slash (for scoped packages)
static SAFE_PACKAGE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9@._+/-]+$").expect("Invalid regex pattern"));

/// Characters that could be dangerous in shell contexts
static SHELL_DANGEROUS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[;`$(){}|&<>\\'"\n\r\t]"#).expect("Invalid regex pattern"));

/// Validate a package name is safe for shell execution
///
/// # Security
/// Prevents command injection by ensuring package names only contain
/// safe characters. Package names like `foo; rm -rf /` will be rejected.
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(DeclarchError::ConfigError(
            "Package name cannot be empty".to_string(),
        ));
    }

    if name.len() > 256 {
        return Err(DeclarchError::ConfigError(format!(
            "Package name too long (max 256 chars): {}",
            &name[..50]
        )));
    }

    // Check for dangerous shell characters
    if SHELL_DANGEROUS.is_match(name) {
        return Err(DeclarchError::ConfigError(format!(
            "Package name contains unsafe characters: {}",
            name
        )));
    }

    // Validate against safe pattern
    if !SAFE_PACKAGE_NAME.is_match(name) {
        return Err(DeclarchError::ConfigError(format!(
            "Package name contains invalid characters: {}",
            name
        )));
    }

    // Prevent path traversal
    if name.contains("..") {
        return Err(DeclarchError::ConfigError(format!(
            "Package name cannot contain path traversal: {}",
            name
        )));
    }

    Ok(())
}

/// Validate a list of package names
pub fn validate_package_names(names: &[String]) -> Result<()> {
    for name in names {
        validate_package_name(name)?;
    }
    Ok(())
}

/// Sanitize a hook command for display (log purposes only)
/// This does NOT make a command safe to execute
pub fn sanitize_for_display(input: &str) -> String {
    // Truncate long commands
    if input.len() > 200 {
        format!("{}...", &input[..200])
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(validate_package_name("hyprland").is_ok());
        assert!(validate_package_name("hyprland-git").is_ok());
        assert!(validate_package_name("python3.12").is_ok());
        assert!(validate_package_name("@angular/cli").is_ok());
        assert!(validate_package_name("node_modules").is_ok());
        assert!(validate_package_name("com.spotify.Client").is_ok());
        assert!(validate_package_name("gcc++").is_ok());
    }

    #[test]
    fn test_shell_injection_blocked() {
        // Semicolon injection
        assert!(validate_package_name("foo; rm -rf /").is_err());
        // Pipe injection
        assert!(validate_package_name("foo | cat").is_err());
        // Command substitution
        assert!(validate_package_name("foo$(cat)").is_err());
        // Ampersand chaining
        assert!(validate_package_name("foo && echo").is_err());
    }

    #[test]
    fn test_path_traversal_blocked() {
        assert!(validate_package_name("../../../etc/passwd").is_err());
        assert!(validate_package_name("foo/../bar").is_err());
    }

    #[test]
    fn test_empty_and_long_names() {
        assert!(validate_package_name("").is_err());
        let long_name = "a".repeat(300);
        assert!(validate_package_name(&long_name).is_err());
    }
}
