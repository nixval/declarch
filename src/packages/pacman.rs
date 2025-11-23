use crate::package::trait_impl::{PackageManager, InstalledPackage};
use crate::utils::errors::{DeclarchError, Result};
use std::process::{Command, Stdio};

pub struct PacmanManager;

impl PackageManager for PacmanManager {
    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }
        
        let status = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .arg("--needed")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "pacman".to_string(), 
                reason: e.to_string() 
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError { reason: "Pacman install failed".to_string() });
        }
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() { return Ok(()); }

        let status = Command::new("sudo")
            .arg("pacman")
            .arg("-Rns")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed { 
                command: "pacman -Rns".to_string(), 
                reason: e.to_string() 
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError { reason: "Pacman remove failed".to_string() });
        }
        Ok(())
    }

    fn check(&self, package: &str) -> Result<bool> {
        let status = Command::new("pacman")
            .arg("-Qi")
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
        "pacman"
    }
}
