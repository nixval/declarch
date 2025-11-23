use crate::packages::traits::PackageManager;
use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use chrono::Utc;

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
            .args(&["list", "--app", "--columns=application,version"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "flatpak list".into(), 
                reason: e.to_string() 
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list flatpak packages".into()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
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
            return Err(DeclarchError::PackageManagerError("Flatpak install failed".into()));
        }
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }

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
            return Err(DeclarchError::PackageManagerError("Flatpak remove failed".into()));
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("flatpak").is_ok()
    }
}
