use super::resolve_target_path;
use tempfile::tempdir;

#[test]
fn resolve_target_rejects_parent_dir_traversal() {
    let dir = tempdir().expect("tempdir");
    let err = resolve_target_path(dir.path(), "../evil").expect_err("must reject traversal");
    assert!(err.to_string().contains("traversal"));
}

#[test]
fn resolve_target_accepts_nested_module_path() {
    let dir = tempdir().expect("tempdir");
    let path = resolve_target_path(dir.path(), "hyprland/niri").expect("valid path");
    assert!(path.ends_with("modules/hyprland/niri.kdl"));
}
