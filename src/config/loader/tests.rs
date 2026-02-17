
use super::*;

#[test]
fn filter_keeps_default_nodes_without_selectors() {
    let content = r#"
pkg { aur { git } }
profile "desktop" {
  pkg { aur { hyprland } }
}
"#;
    let out = filter_content_by_selectors(content, &LoadSelectors::default()).unwrap();
    assert!(out.contains("profile \"desktop\""));
    assert!(out.contains("pkg"));
}

#[test]
fn filter_expands_selected_profile_and_host() {
    let content = r#"
pkg { aur { git } }
profile "desktop" {
  pkg { aur { hyprland } }
}
host "vps-1" {
  pkg { aur { tmux } }
}
"#;
    let out = filter_content_by_selectors(
        content,
        &LoadSelectors {
            profile: Some("desktop".to_string()),
            host: Some("vps-1".to_string()),
        },
    )
    .unwrap();

    assert!(out.contains("hyprland"));
    assert!(out.contains("tmux"));
    assert!(out.contains("git"));
    assert!(!out.contains("profile \"desktop\""));
    assert!(!out.contains("host \"vps-1\""));
}
