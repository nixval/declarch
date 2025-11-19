use crate::package::trait_impl::{PackageManager, InstalledPackage};
use crate::utils::errors::{DeclarchError, Result};
use std::process::{Command, Stdio};

pub struct FlatpakManager;

impl PackageManager for FlatpakManager {
    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }
        
        let status = Command::new("flatpak")
            .arg("install")
            .arg("--user")
            .arg("-y")
            .arg("flathub")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "flatpak".to_string(), 
                reason: e.to_string() 
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError { reason: "Flatpak install failed".to_string() });
        }
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }

        let status = Command::new("flatpak")
            .arg("uninstall")
            .arg("--user")
            .arg("-y")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "flatpak uninstall".to_string(), 
                reason: e.to_string() 
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError { reason: "Flatpak uninstall failed".to_string() });
        }
        Ok(())
    }

    fn check(&self, package: &str) -> Result<bool> {
        let status = Command::new("flatpak")
            .arg("info")
            .arg("--user")
            .arg(package)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(s) => Ok(s.success()),
            Err(_) => Ok(false),
        }
    }

    fn get_installed(&self) -> Result<Vec<InstalledPackage>> {
        Ok(vec![])
    }

    fn name(&self) -> &'static str {
        "flatpak"
    }
}
