use crate::error::Result;
use std::process::Command;

/// Build a shell command in a platform-aware way.
///
/// - Unix: `sh -c <command>` or `sudo sh -c <command>`
/// - Windows: `cmd /C <command>` (elevated shell not yet supported)
pub fn build_shell_command(command: &str, elevated: bool) -> Result<Command> {
    #[cfg(unix)]
    {
        let cmd = if elevated {
            let mut c = Command::new("sudo");
            c.arg("sh").arg("-c").arg(command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        return Ok(cmd);
    }

    #[cfg(windows)]
    {
        if elevated {
            return Err(DeclarchError::Other(
                "Elevated shell execution is not implemented for Windows yet".to_string(),
            ));
        }

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command);
        return Ok(cmd);
    }

    #[cfg(not(any(unix, windows)))]
    {
        if elevated {
            return Err(DeclarchError::Other(
                "Elevated shell execution is not implemented on this platform".to_string(),
            ));
        }

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        Ok(cmd)
    }
}

/// Build a direct program invocation in a platform-aware way.
///
/// - Unix: `program args...` or `sudo program args...`
/// - Windows: `program args...` (elevated direct execution not yet supported)
pub fn build_program_command(program: &str, args: &[String], elevated: bool) -> Result<Command> {
    #[cfg(unix)]
    {
        let cmd = if elevated {
            let mut c = Command::new("sudo");
            c.arg(program);
            c.args(args);
            c
        } else {
            let mut c = Command::new(program);
            c.args(args);
            c
        };

        return Ok(cmd);
    }

    #[cfg(windows)]
    {
        if elevated {
            return Err(DeclarchError::Other(
                "Elevated direct execution is not implemented for Windows yet".to_string(),
            ));
        }

        let mut cmd = Command::new(program);
        cmd.args(args);
        return Ok(cmd);
    }

    #[cfg(not(any(unix, windows)))]
    {
        if elevated {
            return Err(DeclarchError::Other(
                "Elevated direct execution is not implemented on this platform".to_string(),
            ));
        }

        let mut cmd = Command::new(program);
        cmd.args(args);
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_program_non_elevated_works() {
        let cmd = build_program_command("echo", &["ok".to_string()], false).unwrap();
        let debug = format!("{:?}", cmd);
        assert!(debug.contains("echo"));
    }

    #[test]
    fn build_shell_non_elevated_works() {
        let cmd = build_shell_command("echo ok", false).unwrap();
        let debug = format!("{:?}", cmd);
        #[cfg(unix)]
        assert!(debug.contains("\"sh\""));
    }
}
