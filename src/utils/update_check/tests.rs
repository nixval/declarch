use super::*;
use super::{cache_policy, fetcher, owner_detection, versioning};

#[test]
fn compare_versions_works_for_semver() {
    assert!(compare_versions("0.8.3", "0.8.2").is_gt());
    assert!(compare_versions("0.8.2", "0.8.2").is_eq());
    assert!(compare_versions("0.8.2", "0.8.3").is_lt());
}

#[test]
fn parses_prefixed_versions() {
    assert_eq!(versioning::version_tuple("v0.8.2"), Some((0, 8, 2)));
    assert_eq!(versioning::version_tuple("0.8.2-beta.1"), Some((0, 8, 2)));
}

#[test]
fn recognizes_script_like_install_channels() {
    assert!(owner_detection::install_channel_is_script_like(
        r#"{"channel":"script","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(owner_detection::install_channel_is_script_like(
        r#"{"channel":"curl","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(owner_detection::install_channel_is_script_like(
        r#"{"channel":"wget","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(owner_detection::install_channel_is_script_like(
        r#"{"channel":"manual","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
    assert!(!owner_detection::install_channel_is_script_like(
        r#"{"channel":"aur","installed_at":"2026-01-01T00:00:00Z"}"#
    ));
}

#[test]
fn invalid_release_payload_returns_none() {
    assert!(fetcher::parse_latest_version_from_body("not-json").is_none());
    assert!(fetcher::parse_latest_version_from_body(r#"{"tag_name":"latest"}"#).is_none());
}

#[test]
fn stale_cache_prefers_new_fetch_result() {
    let stale_cache = cache_policy::UpdateCache {
        checked_at_unix: 100,
        latest_version: "0.8.2".to_string(),
    };

    let picked = cache_policy::pick_latest_version(200, 24, Some(&stale_cache), Some("0.9.0"));
    assert_eq!(picked.as_deref(), Some("0.9.0"));
}

#[test]
fn offline_mode_falls_back_to_cached_latest() {
    let stale_cache = cache_policy::UpdateCache {
        checked_at_unix: 100,
        latest_version: "0.8.3".to_string(),
    };

    let picked = cache_policy::pick_latest_version(200, 24, Some(&stale_cache), None);
    assert_eq!(picked.as_deref(), Some("0.8.3"));
}
