use super::{CommandMode, GenericManager};
use crate::backends::generic::command_exec::{
    run_command_with_timeout, run_interactive_command_with_timeout,
};
use crate::error::{DeclarchError, Result};
use crate::utils::sanitize;
use std::process::{Command, ExitStatus, Output};
use std::time::Duration;

impl GenericManager {
    /// Get the actual binary to use (first available from alternatives).
    /// Handles fallback if primary binary not available and fallback configured.
    pub(super) fn get_binary(&self) -> Result<String> {
        if let Some(bin) = self.config.binary.find_available() {
            return Ok(bin);
        }

        if let Some(fallback_name) = &self.config.fallback {
            let all_backends = crate::backends::load_all_backends_unified().map_err(|e| {
                DeclarchError::PackageManagerError(format!("Failed to load backend configs: {}", e))
            })?;

            if let Some(fallback_config) = all_backends.get(fallback_name)
                && let Some(fallback_bin) = fallback_config.binary.find_available()
            {
                return Ok(fallback_bin);
            }
        }

        Err(DeclarchError::PackageManagerError(format!(
            "{} not found. Please install {} first.",
            self.config.binary.primary(),
            self.config.name
        )))
    }

    /// Build command with optional sudo.
    /// Uses the resolved binary (respecting fallback if needed).
    pub(super) fn build_command(&self, cmd_str: &str, mode: CommandMode) -> Result<Command> {
        let binary = self.get_binary()?;
        let cmd_str = self.replace_common_placeholders(cmd_str, &binary);

        let use_sudo = self.config.needs_sudo && matches!(mode, CommandMode::Mutating);
        let mut cmd = crate::utils::platform::build_shell_command(&cmd_str, use_sudo)?;

        if let Some(env_vars) = &self.config.preinstall_env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        Ok(cmd)
    }

    /// Format package list for command.
    ///
    /// SECURITY: Each package name is shell-escaped to prevent injection attacks.
    pub(super) fn format_packages(&self, packages: &[String]) -> String {
        packages
            .iter()
            .map(|p| sanitize::shell_escape(p))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Execute non-interactive command and normalize runtime errors.
    pub(super) fn run_output_command(
        &self,
        cmd: &mut Command,
        command_label: &str,
        timeout: Duration,
    ) -> Result<Output> {
        run_command_with_timeout(cmd, timeout).map_err(|e| DeclarchError::SystemCommandFailed {
            command: command_label.to_string(),
            reason: e.to_string(),
        })
    }

    /// Execute interactive command and normalize runtime errors.
    pub(super) fn run_interactive_status(
        &self,
        cmd: &mut Command,
        command_label: &str,
        timeout: Duration,
    ) -> Result<ExitStatus> {
        run_interactive_command_with_timeout(cmd, timeout).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: command_label.to_string(),
                reason: e.to_string(),
            }
        })
    }

    fn format_sources(&self) -> String {
        self.config
            .package_sources
            .as_ref()
            .map(|sources| {
                sources
                    .iter()
                    .map(|s| sanitize::shell_escape(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }

    fn replace_common_placeholders(&self, template: &str, binary: &str) -> String {
        template
            .replace("{binary}", binary)
            .replace("{repos}", &self.format_sources())
    }
}
