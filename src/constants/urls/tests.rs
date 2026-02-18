use super::*;
use crate::project_identity;

#[test]
fn test_build_urls_github() {
    let builder = RemoteUrlBuilder::default();
    let urls = builder.build_urls("myuser/dotfiles");

    // Check that we get some URL with github.com (from default registry)
    // OR declarch-packages registry
    assert!(!urls.is_empty());
    assert!(
        urls.iter()
            .any(|u| u.contains(project_identity::CONFIG_FILE_BASENAME))
    );
}

#[test]
fn test_build_urls_variant() {
    let builder = RemoteUrlBuilder::default();
    let urls = builder.build_urls("hyprland/niri-nico");

    // Check that we get a URL containing variant
    assert!(urls.iter().any(|u| u.contains("niri-nico")));
}

#[test]
fn test_build_urls_direct() {
    let builder = RemoteUrlBuilder::default();
    let urls = builder.build_urls("https://example.com/config.kdl");

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/config.kdl");
}
