use super::*;

#[test]
fn test_parse_package_string_simple() {
    let (backend, package) = parse_package_string("vim").unwrap();
    assert!(backend.is_none());
    assert_eq!(package, "vim");
}

#[test]
fn test_parse_package_string_with_backend() {
    let (backend, package) = parse_package_string("soar:bat").unwrap();
    assert_eq!(backend, Some("soar".to_string()));
    assert_eq!(package, "bat");
}

#[test]
fn test_parse_package_string_npm() {
    let (backend, package) = parse_package_string("npm:node").unwrap();
    assert_eq!(backend, Some("npm".to_string()));
    assert_eq!(package, "node");
}

#[test]
fn test_parse_package_string_invalid_backend_chars() {
    // Backend with invalid characters (space, special chars) should fail
    let result = parse_package_string("invalid backend:package");
    assert!(result.is_err());
    let result = parse_package_string("invalid@backend:package");
    assert!(result.is_err());
}

#[test]
fn test_validate_empty_string() {
    assert!(parse_package_string("").is_err());
}

#[test]
fn test_validate_whitespace_only() {
    assert!(parse_package_string("   ").is_err());
}

#[test]
fn test_validate_colon_only() {
    assert!(parse_package_string(":").is_err());
}

#[test]
fn test_validate_multiple_colons() {
    assert!(parse_package_string("aur::bat").is_err());
    assert!(parse_package_string("::bat").is_err());
    assert!(parse_package_string("aur::").is_err());
}

#[test]
fn test_validate_backend_empty() {
    assert!(parse_package_string(":bat").is_err());
}

#[test]
fn test_validate_package_empty() {
    assert!(parse_package_string("aur:").is_err());
}

#[test]
fn test_validate_trims_whitespace() {
    let (backend, package) = parse_package_string("  aur:bat  ").unwrap();
    assert_eq!(backend, Some("aur".to_string()));
    assert_eq!(package, "bat");
}

#[test]
fn test_is_valid_backend() {
    // Valid backends: alphanumeric + hyphens
    assert!(is_valid_backend("aur"));
    assert!(is_valid_backend("soar"));
    assert!(is_valid_backend("flatpak"));
    assert!(is_valid_backend("npm"));
    assert!(is_valid_backend("my-custom-backend"));
    assert!(is_valid_backend("custom123"));

    // Invalid: empty or contains invalid characters
    assert!(!is_valid_backend(""));
    assert!(!is_valid_backend("invalid backend")); // space
    assert!(!is_valid_backend("invalid@backend")); // special char
    assert!(!is_valid_backend("invalid_backend")); // underscore
}

#[test]
fn test_add_package_to_empty_content() {
    let editor = ConfigEditor::new();
    let content = "";
    let result = editor.add_package_to_content(content, "bat", Some("soar"));

    assert!(result.is_ok());
    let (updated, added) = result.unwrap();

    // Verify package was added
    assert_eq!(added, vec!["bat"]);
    assert!(updated.contains("pkg"));
    assert!(updated.contains("soar"));
    assert!(updated.contains("bat"));
    // Verify valid KDL syntax (has braces)
    assert!(updated.contains('{'));
    assert!(updated.contains('}'));
}

#[test]
fn test_add_package_to_existing_block() {
    let editor = ConfigEditor::new();
    let content = "pkg {\n  soar {\n    vim\n  }\n}\n";
    let result = editor.add_package_to_content(content, "bat", Some("soar"));

    assert!(result.is_ok());
    let (updated, added) = result.unwrap();

    // Verify both packages exist
    assert_eq!(added, vec!["bat"]);
    assert!(updated.contains("vim"));
    assert!(updated.contains("bat"));
    // Verify it's still valid KDL with nested structure
    assert!(updated.contains("pkg"));
    assert!(updated.contains("soar"));
}

#[test]
fn test_prevent_duplicates() {
    let editor = ConfigEditor::new();
    let content = "pkg {\n  soar {\n    bat\n  }\n}\n";
    let result = editor.add_package_to_content(content, "bat", Some("soar"));

    assert!(result.is_ok());
    let (updated, added) = result.unwrap();

    // Should return unchanged - no duplicate
    assert!(added.is_empty());
    assert_eq!(updated, content);
}

#[test]
fn test_add_to_default_pkg_block() {
    let editor = ConfigEditor::new();
    let content = "pkg {\n  nano\n}\n";
    let result = editor.add_package_to_content(content, "vim", None);

    assert!(result.is_ok());
    let (updated, added) = result.unwrap();

    // Verify both packages exist in default block
    assert_eq!(added, vec!["vim"]);
    assert!(updated.contains("nano"));
    assert!(updated.contains("vim"));
    assert!(updated.contains("pkg {"));
}

#[test]
fn test_add_multiple_backends() {
    let editor = ConfigEditor::new();
    let mut content = "";
    let expected_default_backend = detect_default_backend();

    // Add AUR package (no specific backend = uses aur as default)
    let result = editor.add_package_to_content(content, "bat", None);
    assert!(result.is_ok());
    let (updated, _) = result.unwrap();
    content = &updated;

    // Add Soar package
    let result = editor.add_package_to_content(content, "fd", Some("soar"));
    assert!(result.is_ok());
    let (updated, _) = result.unwrap();

    // Verify nested structure: pkg { aur { bat } soar { fd } }
    assert!(updated.contains("pkg"));
    assert!(updated.contains(expected_default_backend));
    assert!(updated.contains("soar"));
    assert!(updated.contains("bat"));
    assert!(updated.contains("fd"));
}
