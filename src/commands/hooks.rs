mod execution;
mod presentation;

use crate::config::kdl::{LifecycleAction, LifecycleConfig, LifecyclePhase};
use crate::error::Result;
use crate::ui as output;
use execution::execute_single_hook;
use presentation::{display_hooks, show_disabled_hooks_warning};

/// Execute hooks for a specific phase
pub fn execute_hooks_by_phase(
    hooks: &Option<LifecycleConfig>,
    phase: LifecyclePhase,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    let hooks = match hooks {
        Some(h) => h,
        None => return Ok(()),
    };

    // Filter hooks by phase
    let phase_hooks: Vec<_> = hooks.actions.iter().filter(|h| h.phase == phase).collect();

    if phase_hooks.is_empty() {
        return Ok(());
    }

    let phase_name = format!("{:?}", phase);
    execute_hooks(&phase_hooks, &phase_name, hooks_enabled, dry_run)
}

/// Execute a list of hooks
pub fn execute_hooks(
    hooks: &[&LifecycleAction],
    phase_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    if hooks.is_empty() {
        return Ok(());
    }

    // Header logic
    output::separator();
    if !hooks_enabled && !dry_run {
        show_disabled_hooks_warning(hooks, phase_name);
        return Ok(());
    }

    // Execution logic
    display_hooks(hooks, &format!("Executing {} Hooks", phase_name), false);

    if dry_run {
        output::info("Dry-run: Hooks not executed");
        return Ok(());
    }

    for hook in hooks {
        execute_single_hook(hook)?;
    }

    Ok(())
}

/// Helper to execute pre-sync hooks
pub fn execute_pre_sync(
    hooks: &Option<LifecycleConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_hooks_by_phase(hooks, LifecyclePhase::PreSync, hooks_enabled, dry_run)
}

/// Helper to execute post-sync hooks
pub fn execute_post_sync(
    hooks: &Option<LifecycleConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_hooks_by_phase(hooks, LifecyclePhase::PostSync, hooks_enabled, dry_run)
}

/// Helper to execute on-success hooks
pub fn execute_on_success(
    hooks: &Option<LifecycleConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_hooks_by_phase(hooks, LifecyclePhase::OnSuccess, hooks_enabled, dry_run)
}

/// Helper to execute on-failure hooks
pub fn execute_on_failure(
    hooks: &Option<LifecycleConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_hooks_by_phase(hooks, LifecyclePhase::OnFailure, hooks_enabled, dry_run)
}

/// Helper to execute on-update hooks
pub fn execute_on_update(
    hooks: &Option<LifecycleConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_hooks_by_phase(hooks, LifecyclePhase::OnUpdate, hooks_enabled, dry_run)
}

fn execute_package_phase(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    phase: LifecyclePhase,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    let hooks = match hooks {
        Some(h) => h,
        None => return Ok(()),
    };

    // Filter hooks by phase and package
    let package_hooks: Vec<_> = hooks
        .actions
        .iter()
        .filter(|h| h.phase == phase)
        .filter(|h| h.package.as_deref().is_none() || h.package.as_deref() == Some(package_name))
        .collect();

    if package_hooks.is_empty() {
        return Ok(());
    }

    execute_hooks(
        &package_hooks,
        &format!("{:?} ({})", phase, package_name),
        hooks_enabled,
        dry_run,
    )
}

/// Helper to execute pre-install hooks for a specific package
pub fn execute_pre_install(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_package_phase(
        hooks,
        package_name,
        LifecyclePhase::PreInstall,
        hooks_enabled,
        dry_run,
    )
}

/// Helper to execute post-install hooks for a specific package
pub fn execute_post_install(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_package_phase(
        hooks,
        package_name,
        LifecyclePhase::PostInstall,
        hooks_enabled,
        dry_run,
    )
}

/// Helper to execute pre-remove hooks for a specific package
pub fn execute_pre_remove(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_package_phase(
        hooks,
        package_name,
        LifecyclePhase::PreRemove,
        hooks_enabled,
        dry_run,
    )
}

/// Helper to execute post-remove hooks for a specific package
pub fn execute_post_remove(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_package_phase(
        hooks,
        package_name,
        LifecyclePhase::PostRemove,
        hooks_enabled,
        dry_run,
    )
}

#[cfg(test)]
mod tests {
    use super::execute_hooks;
    use crate::config::kdl::{ActionType, ErrorBehavior, LifecycleAction, LifecyclePhase};

    fn hook(command: &str, error_behavior: ErrorBehavior) -> LifecycleAction {
        LifecycleAction {
            command: command.to_string(),
            action_type: ActionType::User,
            phase: LifecyclePhase::PreSync,
            package: None,
            conditions: vec![],
            error_behavior,
        }
    }

    #[test]
    fn execute_hooks_dry_run_skips_execution_errors() {
        let h = hook(
            "nonexistent-command-that-would-fail",
            ErrorBehavior::Required,
        );
        let refs = vec![&h];
        let res = execute_hooks(&refs, "PreSync", true, true);
        assert!(res.is_ok());
    }

    #[test]
    fn execute_hooks_required_failure_propagates_error() {
        let h = hook(
            "nonexistent-command-that-will-fail",
            ErrorBehavior::Required,
        );
        let refs = vec![&h];
        let res = execute_hooks(&refs, "PreSync", true, false);
        assert!(res.is_err());
    }
}
