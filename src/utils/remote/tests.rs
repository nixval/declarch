use super::*;

#[test]
fn test_build_urls_github_default() {
    let urls = build_urls("myuser/hyprland1");

    assert!(
        urls.iter()
            .any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl"))
    );
}

#[test]
fn test_build_urls_github_with_branch() {
    let urls = build_urls("myuser/hyprland1/develop");

    assert!(urls.iter().any(|u| {
        u.contains("raw.githubusercontent.com/myuser/hyprland1/develop/declarch.kdl")
    }));
    assert!(
        urls.iter()
            .any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl"))
    );
}

#[test]
fn test_build_urls_gitlab() {
    let urls = build_urls("gitlab.com/user/repo");

    assert!(
        urls.iter()
            .any(|u| u.contains("gitlab.com/user/repo/-/raw/main/declarch.kdl"))
    );
}

#[test]
fn test_build_urls_gitlab_with_branch() {
    let urls = build_urls("gitlab.com/user/repo/develop");

    assert!(
        urls.iter()
            .any(|u| u.contains("gitlab.com/user/repo/-/raw/develop/declarch.kdl"))
    );
}

#[test]
fn test_build_urls_direct_url() {
    let urls = build_urls("https://example.com/config.kdl");

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/config.kdl");
}

#[test]
fn test_build_urls_flat_name() {
    let urls = build_urls("hyprland");

    assert!(
        urls.iter()
            .any(|u| u.contains("declarch-packages/main/modules/hyprland.kdl"))
    );
}

#[test]
fn test_build_urls_default_registry() {
    let urls = build_urls("hyprland/niri-nico");

    // Should try both GitHub and default registry
    assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com")));
    assert!(
        urls.iter()
            .any(|u| u.contains("declarch-packages/main/modules"))
    );
}

#[test]
fn test_build_urls_colon_suffix() {
    let urls = build_urls("myuser/dotfiles:uwsm");

    // Should build URLs with declarch-uwsm.kdl
    assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
    assert!(urls.iter().any(|u| {
        u.contains("raw.githubusercontent.com/myuser/dotfiles/main/declarch-uwsm.kdl")
    }));
}

#[test]
fn test_build_urls_colon_suffix_with_branch() {
    let urls = build_urls("myuser/dotfiles:develop");

    // Should try both develop and main/master branches
    assert!(urls.iter().any(|u| u.contains("declarch-develop.kdl")));
    assert!(
        urls.iter()
            .any(|u| u.contains("/main/declarch-develop.kdl"))
    );
    assert!(
        urls.iter()
            .any(|u| u.contains("/master/declarch-develop.kdl"))
    );
}

#[test]
fn test_build_urls_colon_suffix_main_variant() {
    let urls = build_urls("myuser/dotfiles:main");

    // Should not duplicate - "main" is the default branch
    assert!(urls.iter().any(|u| u.contains("declarch-main.kdl")));
    assert_eq!(urls.len(), 1); // Only one URL since main is default
}

#[test]
fn test_build_urls_registry_with_kdl_extension() {
    let urls = build_urls("gaming/steam-setup.kdl");

    // Registry paths with .kdl should keep the extension
    assert!(
        urls.iter()
            .any(|u| u.contains("declarch-packages/main/modules/gaming/steam-setup.kdl"))
    );
    // Should also try GitHub as fallback
    assert!(
        urls.iter()
            .any(|u| u.contains("raw.githubusercontent.com/gaming/steam-setup/main/declarch.kdl"))
    );
}

#[test]
fn test_build_urls_branch_with_variant() {
    let urls = build_urls("myuser/dotfiles/develop:uwsm");

    // Should build URLs with develop branch and uwsm variant
    assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
    assert!(urls.iter().any(|u| {
        u.contains("raw.githubusercontent.com/myuser/dotfiles/develop/declarch-uwsm.kdl")
    }));
    // Should also fallback to main/master
    assert!(urls.iter().any(|u| u.contains("/main/declarch-uwsm.kdl")));
    assert!(urls.iter().any(|u| u.contains("/master/declarch-uwsm.kdl")));
}

#[test]
fn test_private_address_checks() {
    assert!(is_private_address("localhost"));
    assert!(is_private_address("127.0.0.1"));
    assert!(is_private_address("192.168.1.1"));
    assert!(is_private_address("10.0.0.5"));
    assert!(is_private_address("172.16.0.1"));
    assert!(is_private_address("172.31.255.255"));
    assert!(is_private_address("169.254.169.254")); // Cloud metadata
    assert!(is_private_address("fe80::1")); // IPv6 Link-local
    assert!(is_private_address("fc00::")); // IPv6 ULA

    assert!(!is_private_address("172.32.0.1"));
    assert!(!is_private_address("8.8.8.8"));
    assert!(!is_private_address("1.1.1.1"));
    assert!(!is_private_address("github.com"));
    assert!(!is_private_address("gitlab.com"));
    assert!(!is_private_address("fcdn.example.com"));
}

#[test]
fn test_validate_url_rejects_malformed_host() {
    assert!(validate_url("https://").is_err());
}

#[test]
fn test_validate_url_rejects_http_by_default() {
    assert!(validate_url("http://example.com/config.kdl").is_err());
}

#[test]
fn test_first_private_ip_detects_private_from_resolved_set() {
    let addrs = vec![
        "8.8.8.8".parse::<IpAddr>().expect("parse public ip"),
        "10.0.0.1".parse::<IpAddr>().expect("parse private ip"),
    ];
    assert_eq!(
        first_private_ip(&addrs).expect("private ip expected"),
        "10.0.0.1".parse::<IpAddr>().expect("parse private ip")
    );
}

#[test]
fn test_failure_summary_formats_attempts() {
    let summary = format_failure_summary(&[
        "- u1 => e1".to_string(),
        "- u2 => e2".to_string(),
        "- u3 => e3".to_string(),
        "- u4 => e4".to_string(),
    ]);

    assert!(summary.contains("Fetch attempts:"));
    assert!(summary.contains("u1"));
    assert!(summary.contains("... and 1 more"));
}
