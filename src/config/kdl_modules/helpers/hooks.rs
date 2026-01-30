use crate::config::kdl_modules::types::{
    ActionType, ErrorBehavior, LifecycleAction, LifecycleConfig, LifecyclePhase,
};
use crate::error::{DeclarchError, Result};
use kdl::KdlNode;

/// Parse hooks block: hooks { post-sync { sudo-needed "command" } }
///
/// Supported syntaxes:
/// 1. Global hooks: pre-sync "command"
/// 2. Package hooks (block): docker { post-install "command" --sudo }
/// 3. Package hooks (shorthand): docker:post-install "command" --sudo
pub fn parse_hooks(node: &KdlNode, hooks: &mut LifecycleConfig) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            // Check for shorthand syntax: docker:post-install
            if child_name.contains(':') {
                let parts: Vec<&str> = child_name.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let package = parts[0];
                    let phase_str = parts[1];
                    let phase = parse_hook_phase(phase_str)?;

                    if let Some(command) = super::meta::get_first_string(child) {
                        let (action_type, error_behavior) = parse_hook_flags(child)?;
                        hooks.actions.push(LifecycleAction {
                            command: command.to_string(),
                            action_type,
                            phase,
                            package: Some(package.to_string()),
                            conditions: vec![], // Phase 2
                            error_behavior,
                        });
                    }
                }
            }
            // Check for package block: docker { post-install "..." }
            else if is_package_block(child) {
                let package = child_name.to_string();
                parse_package_hook_block(child, package, hooks)?;
            }
            // Global hooks: pre-sync, post-sync, on-success, on-failure
            else {
                let phase = parse_hook_phase(child_name)?;
                if let Some(command) = super::meta::get_first_string(child) {
                    let (action_type, error_behavior) = parse_hook_flags(child)?;
                    hooks.actions.push(LifecycleAction {
                        command: command.to_string(),
                        action_type,
                        phase,
                        package: None,
                        conditions: vec![], // Phase 2
                        error_behavior,
                    });
                }
            }
        }
    }
    Ok(())
}

/// Parse hook phase from string
pub fn parse_hook_phase(s: &str) -> Result<LifecyclePhase> {
    match s {
        "pre-sync" => Ok(LifecyclePhase::PreSync),
        "post-sync" => Ok(LifecyclePhase::PostSync),
        "on-success" => Ok(LifecyclePhase::OnSuccess),
        "on-failure" => Ok(LifecyclePhase::OnFailure),
        "pre-install" => Ok(LifecyclePhase::PreInstall),
        "post-install" => Ok(LifecyclePhase::PostInstall),
        "pre-remove" => Ok(LifecyclePhase::PreRemove),
        "post-remove" => Ok(LifecyclePhase::PostRemove),
        "on-update" => Ok(LifecyclePhase::OnUpdate),
        _ => Err(DeclarchError::ConfigError(format!(
            "Invalid hook phase '{}'. Valid phases: {}",
            s,
            [
                "pre-sync",
                "post-sync",
                "on-success",
                "on-failure",
                "pre-install",
                "post-install",
                "pre-remove",
                "post-remove",
                "on-update"
            ]
            .join(", ")
        ))),
    }
}

/// Parse hook flags from a node
/// Returns (action_type, error_behavior)
pub fn parse_hook_flags(node: &KdlNode) -> Result<(ActionType, ErrorBehavior)> {
    let mut action_type = ActionType::User;
    let mut error_behavior = ErrorBehavior::default();

    for entry in node.entries().iter().skip(1) {
        // Skip the first entry (command string)
        if let Some(val) = entry.value().as_string() {
            match val {
                "--sudo" => action_type = ActionType::Root,
                "--required" => error_behavior = ErrorBehavior::Required,
                "--ignore" => error_behavior = ErrorBehavior::Ignore,
                _ => {
                    // Unknown flag - could warn here
                }
            }
        }
    }

    Ok((action_type, error_behavior))
}

/// Check if a node is a package block (has children with hook phases)
pub fn is_package_block(node: &KdlNode) -> bool {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            if matches!(
                name,
                "pre-install" | "post-install" | "pre-remove" | "post-remove" | "on-update"
            ) {
                return true;
            }
        }
    }
    false
}

/// Parse package hook block: docker { post-install "..." }
pub fn parse_package_hook_block(
    node: &KdlNode,
    package: String,
    hooks: &mut LifecycleConfig,
) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let phase_str = child.name().value();
            let phase = parse_hook_phase(phase_str)?;

            if let Some(command) = super::meta::get_first_string(child) {
                let (action_type, error_behavior) = parse_hook_flags(child)?;
                hooks.actions.push(LifecycleAction {
                    command: command.to_string(),
                    action_type,
                    phase,
                    package: Some(package.clone()),
                    conditions: vec![], // Phase 2
                    error_behavior,
                });
            }
        }
    }
    Ok(())
}
