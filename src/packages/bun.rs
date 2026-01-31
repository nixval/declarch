//! Bun package manager
//!
//! Installs and manages Node.js packages globally using bun.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct BunManager {
    noconfirm: bool,
}

impl BunManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for BunManager {
    fn backend_type(&self) -> Backend {
        Backend::Bun
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("bun")
            .args(["pm", "ls", "-g"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "bun pm ls -g".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list bun packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse bun pm ls output
        for line in stdout.lines() {
            let line = line.trim();
            // Parse format: package-name@version
            if let Some(at_pos) = line.rfind('@') {
                let name = &line[..at_pos].trim();
                let version = &line[at_pos + 1..].trim();

                if !name.is_empty() && !version.is_empty() {
                    installed.insert(
                        name.to_string(),
                        PackageMetadata {
                            variant: None,
                            version: Some(version.split_whitespace().next().unwrap_or(version).to_string()),
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

        let mut cmd = Command::new("bun");
        cmd.args(["install", "-g"]);

        if self.noconfirm {
            // Bun is non-interactive by default
        }

        cmd.args(packages).stdin(Stdio::inherit()).stdout(Stdio::inherit());

        let status = cmd.status().map_err(|e| DeclarchError::SystemCommandFailed {
            command: "bun install -g".into(),
            reason: e.to_string(),
        })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError("bun install failed".into()));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("bun");
        cmd.args(["remove", "-g"]);

        cmd.args(packages).stdin(Stdio::inherit()).stdout(Stdio::inherit());

        let status = cmd.status().map_err(|e| DeclarchError::SystemCommandFailed {
            command: "bun remove -g".into(),
            reason: e.to_string(),
        })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError("bun remove failed".into()));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("bun").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use npm search since bun doesn't have a built-in search command
        // All npm packages are also bun packages (no limit - let search.rs handle limiting)
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

        let results: Vec<NpmSearchResult> = serde_json::from_str(&stdout)
            .unwrap_or_default();

        let package_results = results
            .into_iter()
            .map(|pkg| PackageSearchResult {
                name: pkg.name,
                version: pkg.version,
                description: pkg.description,
                backend: Backend::Bun,
            })
            .collect();

        Ok(package_results)
    }
}
