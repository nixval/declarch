//! Yarn package manager
//!
//! Installs and manages Node.js packages globally using yarn.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct YarnManager {
    noconfirm: bool,
}

impl YarnManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for YarnManager {
    fn backend_type(&self) -> Backend {
        Backend::Yarn
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("yarn")
            .args(["global", "list", "--depth=0"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "yarn global list".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list yarn packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse yarn global list output:
        // info "package-name@version"
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("info \"") {
                // Extract package@version
                let inner = line
                    .strip_prefix("info \"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap_or(line);

                if let Some(at_pos) = inner.rfind('@') {
                    let name = &inner[..at_pos];
                    let version = &inner[at_pos + 1..];

                    installed.insert(
                        name.to_string(),
                        PackageMetadata {
                            variant: None,
                            version: Some(version.to_string()),
                            installed_at: chrono::Utc::now(),
                            source_file: None,
                        },
                    );
                }
            }
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("yarn");
        cmd.args(["global", "add"]);

        if self.noconfirm {
            cmd.arg("--silent");
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "yarn global add".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "yarn install failed".into(),
            ));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("yarn");
        cmd.args(["global", "remove"]);

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "yarn global remove".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "yarn remove failed".into(),
            ));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("yarn").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use npm search since yarn doesn't have a built-in search command
        // All npm packages are also yarn packages (no limit - let search.rs handle limiting)
        let output = Command::new("npm")
            .args(["search", "--json", query])
            .output()
            .map_err(|e| DeclarchError::Other(format!("npm search failed: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

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
                backend: Backend::Yarn,
            })
            .collect();

        Ok(package_results)
    }
}
