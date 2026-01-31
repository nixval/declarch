//! npm package manager
//!
//! Installs and manages Node.js packages globally using npm.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct NpmManager {
    noconfirm: bool,
}

impl NpmManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for NpmManager {
    fn backend_type(&self) -> Backend {
        Backend::Npm
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("npm")
            .args(["list", "-g", "--depth=0", "--json"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "npm list -g --depth=0 --json".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list npm packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse npm list JSON output
        #[derive(serde::Deserialize)]
        struct NpmListOutput {
            dependencies: Option<HashMap<String, NpmPackage>>,
        }

        #[derive(serde::Deserialize)]
        struct NpmPackage {
            version: String,
        }

        if let Ok(list_output) = serde_json::from_str::<NpmListOutput>(&stdout)
            && let Some(deps) = list_output.dependencies
        {
            for (name, pkg) in deps {
                installed.insert(
                    name.clone(),
                    PackageMetadata {
                        variant: None,
                        version: Some(pkg.version),
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

        let mut cmd = Command::new("npm");
        cmd.arg("install").arg("-g");

        if self.noconfirm {
            cmd.arg("--silent");
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "npm install".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "npm install failed".into(),
            ));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("npm");
        cmd.arg("uninstall").arg("-g");

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "npm uninstall".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "npm uninstall failed".into(),
            ));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("npm").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // npm doesn't have a simple way to get reverse dependencies
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use npm search with JSON output (no limit - let search.rs handle limiting)
        let output = Command::new("npm")
            .args(["search", "--json", query])
            .output()
            .map_err(|e| DeclarchError::Other(format!("npm search failed: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse npm JSON output
        #[derive(serde::Deserialize)]
        struct NpmSearchResult {
            name: String,
            version: Option<String>,
            description: Option<String>,
        }

        let results: Vec<NpmSearchResult> = serde_json::from_str(&stdout).unwrap_or_default();

        let package_results = results
            .into_iter()
            .map(|pkg| PackageSearchResult {
                name: pkg.name,
                version: pkg.version,
                description: pkg.description,
                backend: Backend::Npm,
            })
            .collect();

        Ok(package_results)
    }
}
