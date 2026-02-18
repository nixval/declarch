use crate::config::kdl::{ActionType, ErrorBehavior, LifecycleAction};
use crate::constants::HOOK_TIMEOUT_SECS;
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::sanitize;
use regex::Regex;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

const DEFAULT_HOOK_TIMEOUT: Duration = Duration::from_secs(HOOK_TIMEOUT_SECS);

static SAFE_CHAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_\-.\s/:]+$").expect("Valid regex pattern"));

pub(super) fn execute_single_hook(hook: &LifecycleAction) -> Result<()> {
    validate_hook_command(hook)?;

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

    if use_sudo {
        output::info(&format!("Executing hook with sudo: {}", program));
    } else {
        output::info(&format!("Executing hook: {}", program));
    }

    let mut cmd = crate::utils::platform::build_program_command(program, program_args, use_sudo)?;
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let start_time = Instant::now();
    let timeout = DEFAULT_HOOK_TIMEOUT;

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return handle_hook_error(hook, e, program);
        }
    };

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return handle_hook_status(hook, status);
            }
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    output::warning(&format!(
                        "Hook timed out after {} seconds, killing process...",
                        timeout.as_secs()
                    ));
                    let _ = child.kill();
                    let _ = child.wait();

                    match hook.error_behavior {
                        ErrorBehavior::Required => {
                            return Err(DeclarchError::Other(
                                "Required hook timed out".to_string(),
                            ));
                        }
                        ErrorBehavior::Ignore => return Ok(()),
                        ErrorBehavior::Warn => {
                            output::warning("Hook timed out");
                            return Ok(());
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(DeclarchError::Other(format!(
                    "Failed to wait for hook: {}",
                    e
                )));
            }
        }
    }
}

pub(super) fn validate_hook_command(hook: &LifecycleAction) -> Result<()> {
    let trimmed = hook.command.trim();
    if trimmed.starts_with("sudo ") {
        return Err(DeclarchError::ConfigError(format!(
            "Embedded 'sudo' detected in hook command. Use --sudo flag instead.\n  Command: {}",
            sanitize::sanitize_for_display(&hook.command)
        )));
    }

    if !SAFE_CHAR_REGEX.is_match(&hook.command) {
        return Err(DeclarchError::ConfigError(format!(
            "Hook command contains unsafe characters.\n  Command: {}\n  Allowed: a-zA-Z0-9_-./: and whitespace",
            sanitize::sanitize_for_display(&hook.command)
        )));
    }

    if hook.command.contains("../") || hook.command.contains("..\\") {
        return Err(DeclarchError::ConfigError(
            "Hook command contains path traversal sequence (../)".to_string(),
        ));
    }
    Ok(())
}

fn handle_hook_status(hook: &LifecycleAction, status: std::process::ExitStatus) -> Result<()> {
    if status.success() {
        return Ok(());
    }

    match hook.error_behavior {
        ErrorBehavior::Required => Err(DeclarchError::Other(format!(
            "Required hook failed with status: {}",
            status
        ))),
        ErrorBehavior::Ignore => Ok(()),
        ErrorBehavior::Warn => {
            output::warning(&format!("Hook exited with status: {}", status));
            Ok(())
        }
    }
}

fn handle_hook_error(hook: &LifecycleAction, e: std::io::Error, program: &str) -> Result<()> {
    match hook.error_behavior {
        ErrorBehavior::Required => Err(DeclarchError::Other(format!(
            "Failed to execute hook: {}",
            e
        ))),
        ErrorBehavior::Ignore => Ok(()),
        ErrorBehavior::Warn => {
            if e.kind() == std::io::ErrorKind::NotFound {
                output::warning(&format!("Command not found: {}", program));
            } else {
                output::warning(&format!("Failed to execute hook: {}", e));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_hook_command;
    use crate::config::kdl::{ActionType, ErrorBehavior, LifecycleAction, LifecyclePhase};

    fn hook(cmd: &str) -> LifecycleAction {
        LifecycleAction {
            command: cmd.to_string(),
            action_type: ActionType::User,
            phase: LifecyclePhase::PreSync,
            package: None,
            conditions: vec![],
            error_behavior: ErrorBehavior::Warn,
        }
    }

    #[test]
    fn validate_rejects_embedded_sudo() {
        let err = validate_hook_command(&hook("sudo echo x")).expect_err("sudo should fail");
        assert!(err.to_string().contains("Embedded 'sudo'"));
    }

    #[test]
    fn validate_rejects_path_traversal() {
        let err = validate_hook_command(&hook("bash ../evil.sh")).expect_err("traversal");
        assert!(err.to_string().contains("path traversal"));
    }

    #[test]
    fn validate_accepts_simple_safe_command() {
        validate_hook_command(&hook("echo hello/world")).expect("safe command");
    }
}
