use super::*;

#[test]
fn test_binary_specifier_single() {
    let bin = BinarySpecifier::Single("ls".to_string());
    assert_eq!(bin.primary(), "ls");
}

#[test]
fn test_binary_specifier_multiple() {
    let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
    assert_eq!(bin.primary(), "paru");
}

#[test]
fn test_backend_config_default() {
    let config = BackendConfig::default();
    assert_eq!(config.name, "unknown");
    assert_eq!(config.list_format, OutputFormat::SplitWhitespace);
}
