
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

#[test]
fn test_shell_escape_single_quote() {
    assert_eq!(shell_escape("hello'world"), "'hello'\"'\"'world'");
}

#[test]
fn test_shell_escape_whitespace_and_symbols() {
    assert_eq!(shell_escape("hello world"), "'hello world'");
    assert_eq!(shell_escape("pkg;rm"), "'pkg;rm'");
}

#[test]
fn test_shell_escape_safe_passthrough() {
    assert_eq!(shell_escape("aur:bat@1.0+git"), "aur:bat@1.0+git");
}
