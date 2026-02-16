use crate::error::{DeclarchError, Result};
use crate::ui;
use std::io::Read;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Execute a command with timeout (non-interactive)
pub(super) fn run_command_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<Output> {
    let cmd_debug = format!("{:?}", cmd);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: e.to_string(),
        })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: "Failed to capture stdout".to_string(),
        })?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: "Failed to capture stderr".to_string(),
        })?;

    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::BufReader::new(stdout).read_to_end(&mut buf);
        buf
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::BufReader::new(stderr).read_to_end(&mut buf);
        buf
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();
                    return Err(DeclarchError::SystemCommandFailed {
                        command: cmd_debug,
                        reason: format!("Command timed out after {} seconds", timeout.as_secs()),
                    });
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(DeclarchError::SystemCommandFailed {
                    command: cmd_debug,
                    reason: e.to_string(),
                });
            }
        }
    };

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Execute an interactive command with timeout (shows real-time output)
pub(super) fn run_interactive_command_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> Result<ExitStatus> {
    let cmd_debug = format!("{:?}", cmd);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: e.to_string(),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if start.elapsed() > timeout {
                    ui::warning(&format!(
                        "Command timed out after {} seconds",
                        timeout.as_secs()
                    ));
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DeclarchError::SystemCommandFailed {
                        command: cmd_debug,
                        reason: format!("Command timed out after {} seconds", timeout.as_secs()),
                    });
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(DeclarchError::SystemCommandFailed {
                    command: cmd_debug,
                    reason: e.to_string(),
                });
            }
        }
    }
}
