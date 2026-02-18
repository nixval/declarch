use super::*;

#[test]
fn test_build_variants() {
    let variants = build_variants("hyprland");
    assert!(variants.contains(&"hyprland-git".to_string()));
    assert!(variants.contains(&"hyprland-nightly".to_string()));
    assert!(variants.contains(&"hyprland-bin".to_string()));
}

#[test]
fn test_is_variant() {
    assert!(is_variant("hyprland-git", "hyprland"));
    assert!(is_variant("bat-bin", "bat"));
    assert!(!is_variant("hyprland", "hyprland-git"));
    assert!(!is_variant("bat", "ripgrep"));
}
