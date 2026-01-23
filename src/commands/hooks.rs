use crate::config::kdl::{HookConfig, HookEntry, HookType};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::sanitize;
use colored::Colorize;
use std::process::{Command, Stdio};

/// Execute a list of hooks
pub fn execute_hooks(
    hooks: &[HookEntry],
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

fn display_hooks(hooks: &[HookEntry], title: &str, warn_mode: bool) {
    if warn_mode {
        println!("\n{}:", title.yellow().bold());
    } else {
        println!("\n{}:", title.cyan().bold());
    }

    for hook in hooks {
        let sudo_marker = matches!(hook.hook_type, HookType::SudoNeeded);
        let safe_display = sanitize::sanitize_for_display(&hook.command);
        println!(
            "  {} {}",
            if sudo_marker { "🔒" } else { "→" },
            safe_display.cyan()
        );
    }
}

fn execute_single_hook(hook: &HookEntry) -> Result<()> {
    // Parse the command string into args
    let args = shell_words::split(&hook.command).map_err(|e| {
        DeclarchError::ConfigError(format!(
            "Failed to parse hook command '{}': {}",
            sanitize::sanitize_for_display(&hook.command),
            e
        ))
    })?;

    if args.is_empty() {
        return Ok(());
    }

    let program = &args[0];
    let program_args = &args[1..];

    let use_sudo = matches!(hook.hook_type, HookType::SudoNeeded);

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
            output::warning(&format!("Hook exited with status: {}", status));
            // We verify if we should fail hard or just warn.
            // For now, warn is consistent with previous behavior.
            Ok(())
        }
        Err(e) => {
            // If binary not found, helpful error
            if e.kind() == std::io::ErrorKind::NotFound {
                output::warning(&format!("Command not found: {}", program));
                Ok(()) // Don't crash sync for this
            } else {
                output::warning(&format!("Failed to execute hook: {}", e));
                Ok(())
            }
        }
    }
}

/// Helper to execute pre-sync hooks
pub fn execute_pre_sync(
    hooks: &Option<HookConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    if let Some(h) = hooks {
        execute_hooks(&h.pre_sync, "Pre-sync", hooks_enabled, dry_run)
    } else {
        Ok(())
    }
}

/// Helper to execute post-sync hooks
pub fn execute_post_sync(
    hooks: &Option<HookConfig>,
    hooks_enabled: bool,
    dry_run: bool,
) -> Result<()> {
    if let Some(h) = hooks {
        execute_hooks(&h.post_sync, "Post-sync", hooks_enabled, dry_run)
    } else {
        Ok(())
    }
}
