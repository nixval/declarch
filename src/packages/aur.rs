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
    pub fn new(noconfirm: bool) -> Self {
        let helper = if which::which("paru").is_ok() {
            "paru"
        } else if which::which("yay").is_ok() {
            "yay"
        } else {
            "paru"
        }.to_string();

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

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

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
        if packages.is_empty() { return Ok(()); }
        
        let mut cmd = Command::new(&self.helper_cmd);
        cmd.arg("-S").arg("--needed");

        if self.noconfirm {
            cmd.arg("--noconfirm");
        }

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
        which::which("pacman").is_ok()
    }
}
