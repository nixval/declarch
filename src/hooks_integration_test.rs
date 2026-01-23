// Integration test for hooks system
// This tests the complete hooks functionality

use declarch::config::kdl::{parse_kdl_content, HookPhase, HookType};

#[test]
fn test_hooks_comprehensive() {
    let kdl = r#"
meta {
    description "Test all hooks features"
    author "nixval"
    version "1.0.0"
}

packages {
    bat
}

hooks {
    // Global hooks
    pre-sync "echo 'Pre-sync'"
    post-sync "echo 'Post-sync'"
    on-success "echo 'Success'"
    on-failure "echo 'Failed'"

    // Package hook (block syntax)
    bat {
        post-install "echo 'Bat installed'"
    }

    // Backend hooks
    aur:post-sync "echo 'AUR synced'"
}
"#;

    let config = parse_kdl_content(kdl).unwrap();

    // Test pre-sync hooks
    let pre_sync_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::PreSync)
        .collect();
    assert_eq!(pre_sync_hooks.len(), 1);
    assert_eq!(pre_sync_hooks[0].command, "echo 'Pre-sync'");
    assert!(pre_sync_hooks[0].package.is_none());

    // Test post-sync hooks
    let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::PostSync)
        .collect();
    assert_eq!(post_sync_hooks.len(), 2);
    assert!(post_sync_hooks.iter().any(|h| h.command == "echo 'Post-sync'"));
    assert!(post_sync_hooks.iter().any(|h| h.command == "echo 'AUR synced'"));

    // Test on-success hooks
    let on_success_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::OnSuccess)
        .collect();
    assert_eq!(on_success_hooks.len(), 1);
    assert_eq!(on_success_hooks[0].command, "echo 'Success'");

    // Test on-failure hooks
    let on_failure_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::OnFailure)
        .collect();
    assert_eq!(on_failure_hooks.len(), 1);
    assert_eq!(on_failure_hooks[0].command, "echo 'Failed'");

    // Test post-install hooks
    let post_install_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::PostInstall)
        .collect();
    assert_eq!(post_install_hooks.len(), 1);
    assert_eq!(post_install_hooks[0].command, "echo 'Bat installed'");
    assert_eq!(post_install_hooks[0].package.as_ref().unwrap(), "bat");
}

#[test]
fn test_hooks_flags() {
    let kdl = r#"
hooks {
    post-sync "echo 'Regular hook'"
    post-sync "systemctl restart gdm" --sudo
    nvidia:post-install "mkinitcpio -P" --sudo --required
    post-sync "cleanup.sh" --ignore
}
"#;

    let config = parse_kdl_content(kdl).unwrap();

    let all_hooks = &config.hooks.hooks;

    // Regular hook (no sudo)
    let regular = all_hooks.iter()
        .find(|h| h.command.contains("Regular hook"))
        .unwrap();
    assert_eq!(regular.hook_type, HookType::User);
    assert_eq!(regular.error_behavior, declarch::config::kdl::ErrorBehavior::Warn);

    // Sudo hook
    let sudo = all_hooks.iter()
        .find(|h| h.command.contains("systemctl restart gdm"))
        .unwrap();
    assert_eq!(sudo.hook_type, HookType::Root);

    // Required hook
    let required = all_hooks.iter()
        .find(|h| h.command.contains("mkinitcpio"))
        .unwrap();
    assert_eq!(required.hook_type, HookType::Root);
    assert_eq!(required.error_behavior, declarch::config::kdl::ErrorBehavior::Required);

    // Ignore hook
    let ignore = all_hooks.iter()
        .find(|h| h.command.contains("cleanup.sh"))
        .unwrap();
    assert_eq!(ignore.error_behavior, declarch::config::kdl::ErrorBehavior::Ignore);
}

#[test]
fn test_hooks_shorthand() {
    let kdl = r#"
packages {
    docker
    waybar
}

hooks {
    // Shorthand syntax
    docker:post-install "systemctl enable docker" --sudo
    waybar:post-install "pkill waybar"

    // Backend shorthand
    aur:post-sync "echo 'AUR done'"
    flatpak:post-sync "echo 'Flatpak done'"
}
"#;

    let config = parse_kdl_content(kdl).unwrap();

    // Test package shorthand
    let docker_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.package.as_deref() == Some("docker"))
        .collect();
    assert_eq!(docker_hooks.len(), 1);
    assert_eq!(docker_hooks[0].phase, HookPhase::PostInstall);
    assert_eq!(docker_hooks[0].hook_type, HookType::Root);

    let waybar_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.package.as_deref() == Some("waybar"))
        .collect();
    assert_eq!(waybar_hooks.len(), 1);
    assert_eq!(waybar_hooks[0].phase, HookPhase::PostInstall);

    // Test backend shorthand (should be detected by post-sync phase)
    let aur_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::PostSync)
        .filter(|h| h.command.contains("AUR"))
        .collect();
    assert_eq!(aur_hooks.len(), 1);

    let flatpak_hooks: Vec<_> = config.hooks.hooks.iter()
        .filter(|h| h.phase == HookPhase::PostSync)
        .filter(|h| h.command.contains("Flatpak"))
        .collect();
    assert_eq!(flatpak_hooks.len(), 1);
}
