use crate::packages::traits::PackageManager;
use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::path::Path;
use chrono::Utc;

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Start of ANSI escape sequence
            if chars.next() == Some('[') {
                // Skip until we hit the terminating character
                while let Some(next) = chars.next() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

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
        // Format with ANSI codes: "[○] package#variant:cache | version | type"
        // Format stripped: "[○] gimp#github.com.pkgforge-dev.GIMP-AppImage:pkgcache | 3.0.2-3 | static"
        for line in stdout.lines() {
            // Strip ANSI escape codes
            let clean_line = strip_ansi_codes(line);

            // Skip empty lines
            if clean_line.trim().is_empty() {
                continue;
            }

            // Parse format: [○] name#variant:cache | version | type
            // Extract columns by splitting " | "
            let columns: Vec<&str> = clean_line.split(" | ").collect();
            if columns.len() < 2 {
                continue;
            }

            // First column: "[○] package#variant:cache"
            let first_col = columns[0].trim();

            // Remove status indicator "[○] " prefix
            let package_info = first_col.strip_prefix("[○] ")
                .or_else(|| first_col.strip_prefix("[●] "))  // installed indicator
                .or_else(|| first_col.strip_prefix("[×] "))  // error indicator
                .unwrap_or(first_col);

            // Split package#variant:cache
            // Example: "gimp#github.com.pkgforge-dev.GIMP-AppImage:pkgcache"
            let (base_name, variant) = if let Some(hash_pos) = package_info.find('#') {
                let base = &package_info[..hash_pos];
                let full_variant = &package_info[hash_pos + 1..];
                (base.to_string(), Some(full_variant.to_string()))
            } else {
                // No variant, use whole name as base
                (package_info.to_string(), None)
            };

            // Second column: version
            let version = Some(columns[1].trim().to_string());

            installed.insert(base_name, PackageMetadata {
                version,
                variant,
                installed_at: Utc::now(),
                source_file: None,
            });
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = self.build_command();
        cmd.arg("install");  // Fixed: was "apply", should be "install"

        if self.noconfirm {
            cmd.arg("-y");  // Soar uses -y for yes, not --yes
        }

        // Pass packages as arguments
        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "soar install".into(),
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

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // Soar is a standalone binary manager and doesn't track reverse dependencies
        // Packages are installed independently without tracking what depends on them
        Ok(Vec::new())
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
