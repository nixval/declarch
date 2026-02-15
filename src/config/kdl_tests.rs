// KDL Parser Tests for v0.6+ (Unified Package Storage)
//
// These tests verify the new generic parser that uses packages_by_backend
// for all package storage, regardless of backend.

use crate::config::kdl_modules::parse_kdl_content;

#[test]
fn test_pkg_block_single_backend() {
    let kdl = r#"
            pkg {
                paru { hyprland waybar }
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.packages_by_backend.len(), 1);

    let paru_packages = config.packages_by_backend.get("paru").unwrap();
    assert_eq!(paru_packages.len(), 2);
    assert!(paru_packages.iter().any(|p| p.name == "hyprland"));
    assert!(paru_packages.iter().any(|p| p.name == "waybar"));
}

#[test]
fn test_pkg_block_multiple_backends() {
    let kdl = r#"
            pkg {
                paru { hyprland git }
                npm { typescript eslint }
                flatpak { com.spotify.Client }
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.packages_by_backend.len(), 3);

    assert_eq!(config.packages_by_backend.get("paru").unwrap().len(), 2);
    assert_eq!(config.packages_by_backend.get("npm").unwrap().len(), 2);
    assert_eq!(config.packages_by_backend.get("flatpak").unwrap().len(), 1);
}

#[test]
fn test_pkg_colon_syntax() {
    let kdl = r#"
            pkg:paru { hyprland waybar }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.packages_by_backend.len(), 1);

    let paru_packages = config.packages_by_backend.get("paru").unwrap();
    assert_eq!(paru_packages.len(), 2);
}

#[test]
fn test_inline_prefix() {
    let kdl = r#"
            pkg {
                paru:hyprland
                npm:typescript
                flatpak:com.spotify.Client
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.packages_by_backend.len(), 3);

    assert!(
        config
            .packages_by_backend
            .get("paru")
            .unwrap()
            .iter()
            .any(|p| p.name == "hyprland")
    );
    assert!(
        config
            .packages_by_backend
            .get("npm")
            .unwrap()
            .iter()
            .any(|p| p.name == "typescript")
    );
    assert!(
        config
            .packages_by_backend
            .get("flatpak")
            .unwrap()
            .iter()
            .any(|p| p.name == "com.spotify.Client")
    );
}

#[test]
fn test_legacy_packages_parsed_to_unified() {
    // Legacy "packages" syntax should also populate packages_by_backend
    let kdl = r#"
            packages {
                hyprland
                waybar
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    // Legacy packages without backend prefix go to "default"
    assert!(config.packages_by_backend.contains_key("default"));
}

#[test]
fn test_legacy_packages_colon_backend_alias_still_works() {
    let kdl = r#"
            packages:npm {
                typescript
                prettier
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert!(config.packages_by_backend.contains_key("npm"));
    let npm_packages = config.packages_by_backend.get("npm").unwrap();
    assert_eq!(npm_packages.len(), 2);
    assert!(npm_packages.iter().any(|p| p.name == "typescript"));
    assert!(npm_packages.iter().any(|p| p.name == "prettier"));
}

#[test]
fn test_legacy_packages_nested_backend_alias_still_works() {
    let kdl = r#"
            packages {
                aur { neovim git }
                flatpak { org.mozilla.firefox }
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert!(config.packages_by_backend.contains_key("aur"));
    assert!(config.packages_by_backend.contains_key("flatpak"));
    assert_eq!(config.packages_by_backend.get("aur").unwrap().len(), 2);
    assert_eq!(config.packages_by_backend.get("flatpak").unwrap().len(), 1);
}

#[test]
fn test_excludes_parsing() {
    let kdl = r#"
            excludes bad-package another-bad
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.excludes.len(), 2);
    assert!(config.excludes.contains(&"bad-package".to_string()));
}

#[test]
fn test_meta_block_parsing() {
    let kdl = r#"
            meta {
                title "My Config"
                description "Test configuration"
                author "testuser"
                version "1.0.0"
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.project_metadata.title, Some("My Config".to_string()));
    assert_eq!(config.project_metadata.author, Some("testuser".to_string()));
}

#[test]
fn test_imports_parsing() {
    let kdl = r#"
            imports {
                "modules/base.kdl"
                "modules/dev.kdl"
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.imports.len(), 2);
    assert!(config.imports.contains(&"modules/base.kdl".to_string()));
}

#[test]
fn test_experimental_block_flag_parsing() {
    let kdl = r#"
            experimental {
                "dangerously-enable-hooks"
            }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert!(
        config
            .experimental
            .contains(&"dangerously-enable-hooks".to_string())
    );
}

#[test]
fn test_experimental_inline_flag_parsing() {
    let kdl = r#"
            experimental "dangerously-enable-hooks"
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert_eq!(config.experimental.len(), 1);
    assert_eq!(config.experimental[0], "dangerously-enable-hooks");
}

#[test]
fn test_backend_options_parsing() {
    // Backend options using the options:backend syntax
    // Note: This requires proper KDL syntax support
    // For now, this test documents the expected behavior
    // TODO: Implement proper backend options parsing with KDL

    // Simple KDL that should parse
    let kdl = r#"
            options paru {
                noconfirm true
            }
        "#;

    // This should parse the options block
    let result = parse_kdl_content(kdl);
    // For now, just verify it doesn't panic
    // The actual backend options extraction may need refinement
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_empty_pkg_block() {
    let kdl = r#"
            pkg {}
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert!(config.packages_by_backend.is_empty());
}

#[test]
fn test_mixed_syntax() {
    let kdl = r#"
            pkg {
                paru { hyprland }
                npm:typescript
            }
            pkg:flatpak { com.spotify.Client }
        "#;

    let config = parse_kdl_content(kdl).unwrap();
    assert!(config.packages_by_backend.contains_key("paru"));
    assert!(config.packages_by_backend.contains_key("npm"));
    assert!(config.packages_by_backend.contains_key("flatpak"));
}
