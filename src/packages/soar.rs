use crate::packages::traits::PackageManager;
use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::path::Path;
use chrono::Utc;

pub struct SoarManager {
    pub noconfirm: bool,
    pub config_dir: Option<String>,
}

impl SoarManager {
    pub fn new(noconfirm: bool) -> Self {
        Self {
            noconfirm,
            config_dir: Self::get_config_dir(),
        }
    }

    /// Get Soar configuration directory
    /// Checks XDG_CONFIG_HOME or defaults to ~/.config/soar
    fn get_config_dir() -> Option<String> {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|p| !p.is_empty())
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| format!("{}/.config", h))
            })
            .map(|base| format!("{}/soar", base))
            .filter(|p| Path::new(p).exists())
    }

    /// Check if soar command is available
    fn is_soar_installed() -> bool {
        Path::new("/usr/bin/soar").exists()
            || Path::new("/usr/local/bin/soar").exists()
            || Command::new("which")
                .arg("soar")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    /// Build soar command with common arguments
    fn build_command(&self) -> Command {
        let mut cmd = if Self::is_soar_installed() {
            Command::new("soar")
        } else {
            // Try using flatpak to run soar if direct installation not found
            let mut cmd = Command::new("flatpak");
            cmd.arg("run").arg("com.github.juniorzouai.soar");
            cmd
        };

        // Add config directory if available
        if let Some(ref config_dir) = self.config_dir {
            cmd.env("SOAR_CONFIG_DIR", config_dir);
        }

        cmd
    }
}

impl PackageManager for SoarManager {
    fn backend_type(&self) -> Backend {
        Backend::Soar
    }

    fn is_available(&self) -> bool {
        Self::is_soar_installed()
            || Command::new("flatpak")
                .args(["list", "--app=com.github.juniorzouai.soar"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // Use `soar list` to get installed packages
        let mut cmd = self.build_command();
        cmd.arg("list");

        let output = cmd
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "soar list".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to query Soar package database".into()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse soar list output
        // Expected format: "package-name version"
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(name) = parts.get(0) {
                let version = parts.get(1).map(|&v| v.to_string());

                installed.insert(name.to_string(), PackageMetadata {
                    version,
                    installed_at: Utc::now(),
                    source_file: None,
                });
            }
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = self.build_command();
        cmd.arg("apply");

        if self.noconfirm {
            cmd.arg("--yes");
        }

        // Pass packages as arguments
        // Soar apply accepts package names
        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "soar apply".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                format!("Soar installation failed for packages: {:?}", packages)
            ));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = self.build_command();
        cmd.arg("remove");

        if self.noconfirm {
            cmd.arg("--yes");
        }

        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "soar remove".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                format!("Soar removal failed for packages: {:?}", packages)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soar_manager_creation() {
        let manager = SoarManager::new(false);
        assert_eq!(manager.backend_type(), Backend::Soar);
        assert!(!manager.noconfirm);
    }

    #[test]
    fn test_soar_manager_noconfirm() {
        let manager = SoarManager::new(true);
        assert!(manager.noconfirm);
    }

    #[test]
    fn test_config_dir_detection() {
        let config_dir = SoarManager::get_config_dir();
        // We can't test the exact value since it depends on the system,
        // but we can verify it returns a valid format or None
        match config_dir {
            Some(dir) => assert!(dir.contains("soar") || dir.contains(".config")),
            None => {}, // Valid if config dir doesn't exist
        }
    }
}
