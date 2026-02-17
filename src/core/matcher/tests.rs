
use super::*;
use crate::core::types::Backend;
use chrono::Utc;

fn mock_metadata() -> PackageMetadata {
    PackageMetadata {
        version: Some("1.0.0".to_string()),
        variant: None,
        installed_at: Utc::now(),
        source_file: None,
    }
}

#[test]
fn test_exact_match() {
    let matcher = PackageMatcher::new();
    let mut snapshot = HashMap::new();

    let pkg_id = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::from("aur"),
    };
    snapshot.insert(pkg_id.clone(), mock_metadata());

    let result = matcher.find_package(&pkg_id, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "hyprland");
}

#[test]
fn test_suffix_match() {
    let matcher = PackageMatcher::new();
    let mut snapshot = HashMap::new();

    let git_pkg = PackageId {
        name: "hyprland-git".to_string(),
        backend: Backend::from("aur"),
    };
    snapshot.insert(git_pkg.clone(), mock_metadata());

    let target = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::from("aur"),
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "hyprland-git");
}

#[test]
fn test_prefix_match() {
    let matcher = PackageMatcher::new();
    let mut snapshot = HashMap::new();

    let base_pkg = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::from("aur"),
    };
    snapshot.insert(base_pkg.clone(), mock_metadata());

    let target = PackageId {
        name: "hyprland-git".to_string(),
        backend: Backend::from("aur"),
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "hyprland");
}

#[test]
fn test_is_same_package() {
    let matcher = PackageMatcher::new();

    let pkg1 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::from("aur"),
    };
    let pkg2 = PackageId {
        name: "hyprland-git".to_string(),
        backend: Backend::from("aur"),
    };

    assert!(matcher.is_same_package(&pkg1, &pkg2));
}

#[test]
fn test_is_different_package() {
    let matcher = PackageMatcher::new();

    let pkg1 = PackageId {
        name: "hyprland".to_string(),
        backend: Backend::from("aur"),
    };
    let pkg2 = PackageId {
        name: "wayland".to_string(),
        backend: Backend::from("aur"),
    };

    assert!(!matcher.is_same_package(&pkg1, &pkg2));
}

#[test]
fn test_variant_detection_works_for_any_backend() {
    let matcher = PackageMatcher::new();
    let mut snapshot = HashMap::new();

    // Test with a custom backend (not just aur)
    let git_pkg = PackageId {
        name: "myapp-git".to_string(),
        backend: Backend::from("custom"),
    };
    snapshot.insert(git_pkg.clone(), mock_metadata());

    let target = PackageId {
        name: "myapp".to_string(),
        backend: Backend::from("custom"),
    };

    let result = matcher.find_package(&target, &snapshot);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "myapp-git");
}
