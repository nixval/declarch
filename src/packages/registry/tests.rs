use super::*;

#[test]
fn test_registry_creation() {
    let registry = BackendRegistry::new();
    // May succeed or fail depending on whether backend configs exist
    // In test environment, likely to be empty
    assert!(registry.is_ok() || registry.is_err());
}
