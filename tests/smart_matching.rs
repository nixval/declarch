// Unit tests for smart matching logic
use chrono::Utc;
use declarch::core::matcher::PackageMatcher;
use declarch::core::types::{Backend, PackageId, PackageMetadata};
use std::collections::HashMap;

fn create_snapshot(packages: Vec<(&str, Backend, &str)>) -> HashMap<PackageId, PackageMetadata> {
    let mut snapshot = HashMap::new();
    for (name, backend, version) in packages {
        let id = PackageId {
            name: name.to_string(),
            backend: backend.clone(),
        };
        let meta = PackageMetadata {
            version: Some(version.to_string()),
            variant: None,
            installed_at: Utc::now(),
            source_file: None,
        };
        snapshot.insert(id, meta);
    }
    snapshot
}

#[test]
fn test_smart_match_exact() {
    let snapshot = create_snapshot(vec![
        ("hyprland", Backend::Aur, "0.40.0"),
        ("bat", Backend::Soar, "0.24.0"),
    ]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "hyprland");
}

#[test]
fn test_smart_match_git_variant() {
    let snapshot = create_snapshot(vec![
        ("hyprland-git", Backend::Aur, "0.40.0.r5"),
        ("bat", Backend::Soar, "0.24.0"),
    ]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "hyprland-git");
}

#[test]
fn test_smart_match_bin_variant() {
    let snapshot = create_snapshot(vec![("spotify-bin", Backend::Aur, "1.2.0")]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "spotify".to_string(),
        backend: Backend::Aur,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "spotify-bin");
}

#[test]
fn test_smart_match_prefix_variant() {
    // Tests stripping -git suffix from config to match installed package
    let snapshot = create_snapshot(vec![("python-pipewire", Backend::Aur, "1.0.0")]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "python-pipewire-git".to_string(),
        backend: Backend::Aur,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "python-pipewire");
}

#[test]
fn test_smart_match_flatpak_case_insensitive() {
    let snapshot = create_snapshot(vec![("com.spotify.Client", Backend::Flatpak, "1.2.0")]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "Spotify".to_string(),
        backend: Backend::Flatpak,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "com.spotify.Client");
}

#[test]
fn test_smart_match_no_match() {
    let snapshot = create_snapshot(vec![("hyprland", Backend::Aur, "0.40.0")]);

    let matcher = PackageMatcher::new();
    let target = PackageId {
        name: "wayland".to_string(),
        backend: Backend::Aur,
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_none());
}

#[test]
fn test_is_same_package_exact() {
    let id1 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };
    let id2 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };

    let matcher = PackageMatcher::new();
    assert!(matcher.is_same_package(&id1, &id2));
}

#[test]
fn test_is_same_package_variant() {
    let id1 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };
    let id2 = PackageId {
        name: "hyprland-git".to_string(),
        backend: Backend::Aur,
    };

    let matcher = PackageMatcher::new();
    assert!(matcher.is_same_package(&id1, &id2));
}

#[test]
fn test_is_different_package() {
    let id1 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::Aur,
    };
    let id2 = PackageId {
        name: "wayland".to_string(),
        backend: Backend::Aur,
    };

    let matcher = PackageMatcher::new();
    assert!(!matcher.is_same_package(&id1, &id2));
}

#[test]
fn test_is_different_package_backend() {
    let id1 = PackageId {
        name: "bat".to_string(),
        backend: Backend::Aur,
    };
    let id2 = PackageId {
        name: "bat".to_string(),
        backend: Backend::Soar,
    };

    let matcher = PackageMatcher::new();
    assert!(!matcher.is_same_package(&id1, &id2));
}
