use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use chrono::Utc;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct FlatpakManager {
    pub noconfirm: bool,
}

impl FlatpakManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for FlatpakManager {
    fn backend_type(&self) -> Backend {
        Backend::Flatpak
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,version"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "flatpak list".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list flatpak packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if let Some(name) = parts.first() {
                let version = parts.get(1).map(|&v| v.to_string());

                installed.insert(
                    name.to_string(),
                    PackageMetadata {
                        variant: None,
                        version,
                        installed_at: Utc::now(),
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

        let mut cmd = Command::new("flatpak");
        cmd.arg("install").arg("--user").arg("flathub");

        if self.noconfirm {
            cmd.arg("-y");
        }

        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Flatpak install failed".into(),
            ));
        }
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new("flatpak");
        cmd.arg("uninstall").arg("--user");

        if self.noconfirm {
            cmd.arg("-y");
        }

        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Flatpak remove failed".into(),
            ));
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("flatpak").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // Flatpak apps are self-contained and don't have traditional reverse dependencies
        // Runtime dependencies exist but are tracked differently by flatpak itself
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use flatpak search command (no limit - let search.rs handle limiting)
        let output = Command::new("flatpak")
            .args(["search", query])
            .output()
            .map_err(|e| DeclarchError::Other(format!("flatpak search failed: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();

        // Parse flatpak search output (tab-separated):
        // Name\tDescription\tApplication ID\tVersion\tBranch\tRemote
        // Resonance\tTest and explore web services\tio.github.db_mobile.resonance\t2.0.2\tstable\tflathub
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Split by tab
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                let name = parts[0].to_string();
                let description = if !parts[1].is_empty() {
                    Some(parts[1].to_string())
                } else {
                    None
                };
                let version = Some(parts[3].to_string());

                results.push(PackageSearchResult {
                    name,
                    version,
                    description,
                    backend: Backend::Flatpak,
                });
            }
        }

        Ok(results)
    }
}
