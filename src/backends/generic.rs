use crate::backends::config::BackendConfig;
use crate::backends::parsers;
use crate::core::types::{Backend as CoreBackend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::PackageManager;
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Generic package manager that works with any backend configuration
pub struct GenericManager {
    config: BackendConfig,
    noconfirm: bool,
    backend_type: CoreBackend,
}

impl GenericManager {
    /// Create a new generic manager from configuration
    pub fn from_config(
        mut config: BackendConfig,
        backend_type: CoreBackend,
        noconfirm: bool,
    ) -> Self {
        // Set default columns if not specified
        if config.list_name_col.is_none() {
            config.list_name_col = Some(0);
        }
        if config.list_version_col.is_none() {
            config.list_version_col = Some(1);
        }

        Self {
            config,
            noconfirm,
            backend_type,
        }
    }

    /// Get the actual binary to use (first available from alternatives)
    fn get_binary(&self) -> Result<String> {
        self.config.binary.find_available().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "{} not found. Please install {} first.",
                self.config.binary.primary(),
                self.config.name
            ))
        })
    }

    /// Build command with optional sudo
    fn build_command(&self, cmd_str: &str) -> Result<Command> {
        let _binary = self.get_binary()?;

        // Replace {packages} placeholder if present (for install/remove)
        let cmd_str = cmd_str.to_string();

        if self.config.needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg("sh").arg("-c").arg(cmd_str);
            Ok(cmd)
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(cmd_str);
            Ok(cmd)
        }
    }

    /// Format package list for command
    fn format_packages(&self, packages: &[String]) -> String {
        packages.join(" ")
    }
}

impl PackageManager for GenericManager {
    fn backend_type(&self) -> CoreBackend {
        self.backend_type.clone()
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.config.list_cmd)
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: self.config.list_cmd.clone(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "Failed to list packages for {}: {}",
                self.config.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        parsers::parse_package_list(&output.stdout, &self.config)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        // Security: Validate all package names before shell execution
        sanitize::validate_package_names(packages)?;

        let cmd_template = &self.config.install_cmd;
        let package_list = self.format_packages(packages);
        let mut cmd_str = cmd_template.replace("{packages}", &package_list);

        // Append noconfirm flag if configured and enabled
        if self.noconfirm
            && let Some(flag) = &self.config.noconfirm_flag
        {
            cmd_str.push(' ');
            cmd_str.push_str(flag);
        }

        let mut cmd = self.build_command(&cmd_str)?;

        // Set environment variables if configured
        if let Some(env_vars) = &self.config.preinstall_env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: format!("install: {}", cmd_str),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "{} install failed",
                self.config.name
            )));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        // Security: Validate all package names before shell execution
        sanitize::validate_package_names(packages)?;

        let cmd_template = &self.config.remove_cmd;
        let package_list = self.format_packages(packages);
        let mut cmd_str = cmd_template.replace("{packages}", &package_list);

        // Append noconfirm flag if configured and enabled
        if self.noconfirm
            && let Some(flag) = &self.config.noconfirm_flag
        {
            cmd_str.push(' ');
            cmd_str.push_str(flag);
        }

        let mut cmd = self.build_command(&cmd_str)?;

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: format!("remove: {}", cmd_str),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "{} remove failed",
                self.config.name
            )));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        self.config.binary.find_available().is_some()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // For generic backends, we can't determine dependencies without backend-specific logic
        // Return empty list as default (safe behavior)
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::{BackendConfig, BinarySpecifier};
    use crate::core::types::Backend;

    #[test]
    fn test_format_packages() {
        let config = BackendConfig {
            name: "test".to_string(),
            binary: BinarySpecifier::Single("echo".to_string()),
            ..Default::default()
        };

        let manager = GenericManager::from_config(config, Backend::Aur, false);
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];

        assert_eq!(manager.format_packages(&packages), "pkg1 pkg2");
    }

    #[test]
    fn test_binary_specifier_primary() {
        let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
        assert_eq!(bin.primary(), "paru");
    }
}
