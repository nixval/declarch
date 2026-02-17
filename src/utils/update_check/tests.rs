use super::*;

#[test]
fn compare_versions_works_for_semver() {
    assert!(compare_versions("0.8.2", "0.8.1").is_gt());
    assert!(compare_versions("0.8.1", "0.8.1").is_eq());
    assert!(compare_versions("0.8.0", "0.8.1").is_lt());
}

#[test]
fn parses_prefixed_versions() {
    assert_eq!(version_tuple("v0.8.1"), Some((0, 8, 1)));
    assert_eq!(version_tuple("0.8.1-beta.1"), Some((0, 8, 1)));
}

#[test]
fn recognizes_script_like_install_channels() {
    assert!(install_channel_is_script_like(
        r#"{"channel":"script","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(install_channel_is_script_like(
        r#"{"channel":"curl","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(install_channel_is_script_like(
        r#"{"channel":"wget","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(install_channel_is_script_like(
        r#"{"channel":"manual","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(!install_channel_is_script_like(
        r#"{"channel":"aur","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
}
