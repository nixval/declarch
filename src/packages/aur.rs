use crate::packages::traits::PackageManager;
use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use chrono::Utc;

pub struct AurManager {
    pub helper_cmd: String,
    pub noconfirm: bool,
}

impl AurManager {
    // Now accepts the specific helper command string from config/cli
    pub fn new(helper: String, noconfirm: bool) -> Self {
        Self { 
            helper_cmd: helper,
            noconfirm 
        }
    }
}

impl PackageManager for AurManager {
    fn backend_type(&self) -> Backend {
        Backend::Aur
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // We use pacman -Q for speed. It covers both Repo and AUR.
        let output = Command::new("pacman")
            .arg("-Q")
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "pacman -Q".into(), 
                reason: e.to_string() 
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to query pacman database".into()
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|_| DeclarchError::PackageManagerError(
                "Pacman output contained invalid UTF-8".into()
            ))?;
        let mut installed = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(name) = parts.first() {
                let version = parts.get(1).map(|&v| v.to_string());
                
                installed.insert(name.to_string(), PackageMetadata {
                variant: None,
                    version,
                    installed_at: Utc::now(), 
                    source_file: None,
                });
            }
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }
        
        let mut cmd = Command::new(&self.helper_cmd);
        cmd.arg("-S").arg("--needed");

        if self.noconfirm {
            cmd.arg("--noconfirm");
        }

        // Pass packages to the command
        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(format!("{} install failed", self.helper_cmd)));
        }
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }

        let mut cmd = Command::new("sudo");
        cmd.arg("pacman").arg("-Rns");

        if self.noconfirm {
            cmd.arg("--noconfirm");
        }

        let status = cmd
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError("Pacman remove failed".into()));
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which(&self.helper_cmd).is_ok()
    }

    fn get_required_by(&self, package: &str) -> Result<Vec<String>> {
        // Use pacman -Qi to get the "Required By" field
        let output = Command::new("pacman")
            .arg("-Qi")
            .arg(package)
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: format!("pacman -Qi {}", package),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            // Package not found or error, return empty list
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|_| DeclarchError::PackageManagerError(
                "Pacman output contained invalid UTF-8".into()
            ))?;

        // Parse the "Required By" field
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Required By") {
                // Format: "Required By  : pkg1 pkg2" or "Required By  : None"
                if let Some(deps) = line.split(':').nth(1) {
                    let deps = deps.trim();
                    if deps.is_empty() || deps == "None" {
                        return Ok(Vec::new());
                    }
                    // Split by whitespace and filter empty
                    let required_by: Vec<String> = deps
                        .split_whitespace()
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                    return Ok(required_by);
                }
            }
        }

        // No "Required By" field found
        Ok(Vec::new())
    }
}
