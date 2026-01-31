//! pnpm package manager
//!
//! Installs and manages Node.js packages globally using pnpm.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct PnpmManager {
    noconfirm: bool,
}

impl PnpmManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for PnpmManager {
    fn backend_type(&self) -> Backend {
        Backend::Pnpm
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("pnpm")
            .args(["list", "-g", "--depth=0"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "pnpm list -g".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list pnpm packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse pnpm list output (similar to yarn)
        for line in stdout.lines() {
            let line = line.trim();
            // Parse format: package-name@version
            if let Some(at_pos) = line.rfind('@') {
                let name = &line[..at_pos];
                // Only take the part before @ if it looks like a package name
                if !name.contains(' ') || name.contains('/') {
                    let version = &line[at_pos + 1..];
                    // Clean up version from ANSI codes
                    let version = version.split_whitespace().next().unwrap_or(version);

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

        let mut cmd = Command::new("pnpm");
        cmd.args(["add", "-g"]);

        if self.noconfirm {
            cmd.arg("--silent");
        }

        cmd.args(packages).stdin(Stdio::inherit()).stdout(Stdio::inherit());

        let status = cmd.status().map_err(|e| DeclarchError::SystemCommandFailed {
            command: "pnpm add -g".into(),
            reason: e.to_string(),
        })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError("pnpm install failed".into()));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("pnpm");
        cmd.args(["remove", "-g"]);

        cmd.args(packages).stdin(Stdio::inherit()).stdout(Stdio::inherit());

        let status = cmd.status().map_err(|e| DeclarchError::SystemCommandFailed {
            command: "pnpm remove -g".into(),
            reason: e.to_string(),
        })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError("pnpm remove failed".into()));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("pnpm").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use npm search since pnpm doesn't have a built-in search command
        // All npm packages are also pnpm packages (no limit - let search.rs handle limiting)
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
                backend: Backend::Pnpm,
            })
            .collect();

        Ok(package_results)
    }
}
