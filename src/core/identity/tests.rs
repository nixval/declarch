
use super::*;

#[test]
fn test_identity_from_config() {
    let identity = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));

    assert_eq!(identity.config_name, "hyprland");
    assert_eq!(identity.provides_name, "hyprland");
    assert_eq!(identity.actual_package_name, None);
}

#[test]
fn test_identity_from_installed() {
    let identity = PackageIdentity::from_installed(
        "hyprland".to_string(),
        Some("hyprland-git".to_string()),
        Backend::from("aur"),
    );

    assert_eq!(identity.config_name, "hyprland");
    assert_eq!(identity.provides_name, "hyprland");
    assert_eq!(
        identity.actual_package_name,
        Some("hyprland-git".to_string())
    );
}

#[test]
fn test_is_same_package() {
    let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
    let id2 = PackageIdentity::from_installed(
        "hyprland".to_string(),
        Some("hyprland-git".to_string()),
        Backend::from("aur"),
    );

    assert!(id1.is_same_package(&id2));
}

#[test]
fn test_is_variant_transition() {
    let id1 = PackageIdentity::from_installed(
        "hyprland".to_string(),
        Some("hyprland".to_string()),
        Backend::from("aur"),
    );
    let id2 = PackageIdentity::from_installed(
        "hyprland".to_string(),
        Some("hyprland-git".to_string()),
        Backend::from("aur"),
    );

    assert!(id1.is_variant_transition(&id2));
}

#[test]
fn test_different_packages() {
    let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
    let id2 = PackageIdentity::from_config("wayland".to_string(), Backend::from("aur"));

    assert!(!id1.is_same_package(&id2));
    assert!(!id1.is_variant_transition(&id2));
}

#[test]
fn test_display_name() {
    let id1 = PackageIdentity::from_config("hyprland".to_string(), Backend::from("aur"));
    assert_eq!(id1.display_name(), "hyprland");

    let id2 = PackageIdentity::from_installed(
        "hyprland".to_string(),
        Some("hyprland-git".to_string()),
        Backend::from("aur"),
    );
    assert_eq!(id2.display_name(), "hyprland (installed as hyprland-git)");
}
