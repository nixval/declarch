//! Homebrew package manager
//!
//! Installs and manages packages using Homebrew.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct BrewManager {
    noconfirm: bool,
}

impl BrewManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for BrewManager {
    fn backend_type(&self) -> Backend {
        Backend::Brew
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("brew")
            .args(["list", "--versions"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "brew list --versions".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list brew packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse brew list output:
        // package-name 1.2.3
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(name) = parts.first() {
                let version = parts.get(1).map(|&v| v.to_string());

                installed.insert(
                    name.to_string(),
                    PackageMetadata {
                        variant: None,
                        version,
                        installed_at: chrono::Utc::now(),
                        source_file: None,
                    },
                );
            }
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("brew");
        cmd.arg("install");

        if self.noconfirm {
            // Homebrew doesn't have a --noconfirm flag, it's non-interactive by default
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "brew install".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "brew install failed".into(),
            ));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("brew");
        cmd.arg("uninstall");

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "brew uninstall".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "brew uninstall failed".into(),
            ));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("brew").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // brew doesn't have a simple way to get reverse dependencies
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use brew search command
        let output = Command::new("brew")
            .args(["search", query])
            .output()
            .map_err(|e| DeclarchError::Other(format!("brew search failed: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();

        // Parse brew search output:
        // ==> formula-name
        //     description here
        let mut current_name: Option<String> = None;
        for line in stdout.lines() {
            let line = line.trim();
            if let Some(name_part) = line.strip_prefix("==> ") {
                // Save previous package if exists
                if let Some(name) = current_name.take() {
                    results.push(PackageSearchResult {
                        name,
                        version: None,
                        description: None,
                        backend: Backend::Brew,
                    });
                }
                // Extract new package name
                current_name = Some(name_part.to_string());
            } else if !line.is_empty() && current_name.is_some() {
                // This is likely a description line
                // We could capture it, but for now just move on
            }
        }

        // Don't forget the last package
        if let Some(name) = current_name {
            results.push(PackageSearchResult {
                name,
                version: None,
                description: None,
                backend: Backend::Brew,
            });
        }

        Ok(results)
    }
}
