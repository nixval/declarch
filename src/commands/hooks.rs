use crate::config::kdl::{LifecycleConfig, LifecycleAction, ActionType, LifecyclePhase, ErrorBehavior};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::sanitize;
use colored::Colorize;
use std::process::{Command, Stdio};

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
    let phase_hooks: Vec<_> = hooks.actions.iter()
        .filter(|h| h.phase == phase)
        .collect();

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
            "{} hooks detected but not executed (--hooks not provided)",
            phase_name
        ));
        display_hooks(hooks, phase_name, true);
        println!("\n{}", "To enable hooks, use the --hooks flag:".dimmed());
        println!("  {}", "dc sync --hooks".bold());
        println!(
            "\n{}",
            "Security: Hooks from remote configs may contain arbitrary commands.".yellow()
        );
        println!("{}", "Review the config before enabling hooks.".yellow());
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

fn execute_single_hook(hook: &LifecycleAction) -> Result<()> {
    // Validate: Don't allow embedded "sudo" in command
    let trimmed = hook.command.trim();
    if trimmed.starts_with("sudo ") {
        return Err(DeclarchError::ConfigError(format!(
            "Embedded 'sudo' detected in hook command. Use --sudo flag instead.\n  Command: {}",
            sanitize::sanitize_for_display(&hook.command)
        )));
    }

    // Security: Validate command contains only safe characters before parsing
    // Allow: alphanumeric, spaces, tabs, slashes, dots, hyphens, underscores, @, :, =, $, ~
    let safe_char_regex = regex::Regex::new(r"^[\w\s\-\./@:=\$~\{\}]+$").unwrap();
    if !safe_char_regex.is_match(&hook.command) {
        return Err(DeclarchError::ConfigError(format!(
            "Hook command contains unsafe characters.\n  Command: {}",
            sanitize::sanitize_for_display(&hook.command)
        )));
    }

    // Parse the command string into args using shlex (safer than shell_words)
    let args = shlex::split(&hook.command).ok_or_else(|| {
        DeclarchError::ConfigError(format!(
            "Failed to parse hook command '{}': Invalid quoting or escaping",
            sanitize::sanitize_for_display(&hook.command)
        ))
    })?;

    if args.is_empty() {
        return Ok(());
    }

    let program = &args[0];
    let program_args = &args[1..];

    let use_sudo = matches!(hook.action_type, ActionType::Root);

    let mut cmd = if use_sudo {
        output::info(&format!("Executing hook with sudo: {}", program));
        let mut c = Command::new("sudo");
        // Pass program and args to sudo
        // sudo [program] [arg1] [arg2] ...
        c.arg(program);
        c.args(program_args);
        c
    } else {
        output::info(&format!("Executing hook: {}", program));
        // Run directly without shell wrapper for security
        let mut c = Command::new(program);
        c.args(program_args);
        c
    };

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            // Handle based on error_behavior
            match hook.error_behavior {
                ErrorBehavior::Required => {
                    Err(DeclarchError::Other(format!(
                        "Required hook failed with status: {}",
                        status
                    )))
                }
                ErrorBehavior::Ignore => {
                    Ok(())
                }
                ErrorBehavior::Warn => {
                    output::warning(&format!("Hook exited with status: {}", status));
                    Ok(())
                }
            }
        }
        Err(e) => {
            // Handle based on error_behavior
            match hook.error_behavior {
                ErrorBehavior::Required => {
                    Err(DeclarchError::Other(format!(
                        "Failed to execute hook: {}",
                        e
                    )))
                }
                ErrorBehavior::Ignore => {
                    Ok(())
                }
                ErrorBehavior::Warn => {
                    // If binary not found, helpful error
                    if e.kind() == std::io::ErrorKind::NotFound {
                        output::warning(&format!("Command not found: {}", program));
                    } else {
                        output::warning(&format!("Failed to execute hook: {}", e));
                    }
                    Ok(())
                }
            }
        }
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

/// Helper to execute post-install hooks for a specific package
pub fn execute_post_install(
    hooks: &Option<LifecycleConfig>,
    package_name: &str,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    let hooks = match hooks {
        Some(h) => h,
        None => return Ok(()),
    };

    // Filter hooks by phase and package
    let package_hooks: Vec<_> = hooks.actions.iter()
        .filter(|h| h.phase == LifecyclePhase::PostInstall)
        .filter(|h| h.package.as_deref() == Some(package_name))
        .collect();

    if package_hooks.is_empty() {
        return Ok(());
    }

    execute_hooks(&package_hooks, &format!("Post-install ({})", package_name), hooks_enabled, dry_run)
}
