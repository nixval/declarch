//! Integration tests for backends module
//!
//! These tests verify that the backend loading system works correctly
//! with user-defined backend configurations.

use crate::backends::load_all_backends;
use crate::core::types::Backend;

#[test]
fn test_load_backends_empty_when_no_config() {
    // In test environment, likely no backends configured
    let result = load_all_backends();
    assert!(result.is_ok());
    // Result may be empty or have backends depending on environment
}

#[test]
fn test_backend_from_string() {
    // Test that Backend type works with any string
    let backend = Backend::from("any-backend");
    assert_eq!(backend.name(), "any-backend");

    let backend2 = Backend::from("paru");
    assert_eq!(backend2.name(), "paru");

    let backend3 = Backend::from("my-custom-pm");
    assert_eq!(backend3.name(), "my-custom-pm");
}

#[test]
fn test_backend_case_insensitive() {
    // Backend names should be case-insensitive
    let backend1 = Backend::from("NPM");
    let backend2 = Backend::from("npm");
    let backend3 = Backend::from("Npm");

    assert_eq!(backend1.name(), "npm");
    assert_eq!(backend2.name(), "npm");
    assert_eq!(backend3.name(), "npm");
}
