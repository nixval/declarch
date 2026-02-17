mod execution;

use crate::config::kdl::{
    ActionType, LifecycleAction, LifecycleConfig, LifecyclePhase,
};
use crate::error::Result;
use crate::project_identity;
use crate::ui as output;
use crate::utils::sanitize;
use colored::Colorize;
use execution::execute_single_hook;

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
        output::warning(&format!(
            "{} hooks detected but not executed (either --hooks not provided or blocked by policy)",
            phase_name
        ));
        display_hooks(hooks, phase_name, true);

        println!("\n{}", "⚠️  Security Warning:".yellow().bold());
        println!(
            "{}",
            "   Hooks can execute arbitrary system commands.".yellow()
        );
        println!(
            "{}",
            "   Only enable hooks from sources you trust.".yellow()
        );

        println!("\n{}", "To enable hooks after reviewing:".dimmed());
        println!("  {}", project_identity::cli_with("sync --hooks").bold());
        println!("  {}", "dc sync --hooks".dimmed());

        println!("\n{}", "To review the full config:".dimmed());
        println!(
            "  {}",
            format!(
                "cat ~/.config/{}/{}",
                project_identity::CONFIG_DIR_NAME,
                project_identity::CONFIG_FILE_BASENAME
            )
            .dimmed()
        );

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

fn display_hooks(hooks: &[&LifecycleAction], title: &str, warn_mode: bool) {
    if warn_mode {
        println!("\n{}:", title.yellow().bold());
    } else {
        println!("\n{}:", title.cyan().bold());
    }

    for hook in hooks {
        let sudo_marker = matches!(hook.action_type, ActionType::Root);
        let package_info = if let Some(pkg) = &hook.package {
            format!(" [{}]", pkg.cyan())
        } else {
            String::new()
        };
        let safe_display = sanitize::sanitize_for_display(&hook.command);
        println!(
            "  {} {}{}",
            if sudo_marker { "🔒" } else { "→" },
            safe_display.cyan(),
            package_info
        );
    }
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
