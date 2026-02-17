use super::*;

#[test]
fn stable_tool_name_uses_stable_project_id() {
    assert_eq!(
        stable_tool_name("sync_apply"),
        format!("{}_sync_apply", project_identity::STABLE_PROJECT_ID)
    );
}

#[test]
fn normalize_tool_name_with_ids_maps_binary_prefix_to_stable_prefix() {
    let out = normalize_tool_name_with_ids(
        "nextsync_sync_apply",
        project_identity::STABLE_PROJECT_ID,
        "nextsync",
    );
    assert_eq!(
        out,
        format!("{}_sync_apply", project_identity::STABLE_PROJECT_ID)
    );
}

#[test]
fn normalize_tool_name_with_ids_keeps_unknown_names() {
    let out = normalize_tool_name_with_ids(
        "other_tool",
        project_identity::STABLE_PROJECT_ID,
        "nextsync",
    );
    assert_eq!(out, "other_tool");
}
