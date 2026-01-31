//! Cargo package manager
//!
//! Installs and manages Rust crates using cargo.

use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct CargoManager {
    noconfirm: bool,
}

impl CargoManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }
}

impl PackageManager for CargoManager {
    fn backend_type(&self) -> Backend {
        Backend::Cargo
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "cargo install --list".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to list cargo packages".into(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        // Parse cargo list output:
        // package-name v0.1.0:
        for line in stdout.lines() {
            let line = line.trim();
            if let Some(colon_pos) = line.rfind(':') {
                let name_version = &line[..colon_pos];
                if let Some(space_pos) = name_version.rfind(' ') {
                    let name = &name_version[..space_pos];
                    let version = &name_version[space_pos + 1..];

                    // Strip 'v' prefix from version if present
                    let version = version.strip_prefix('v').unwrap_or(version);

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

        let mut cmd = Command::new("cargo");
        cmd.arg("install");

        if self.noconfirm {
            // Cargo doesn't have a noconfirm flag, it's non-interactive by default
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "cargo install".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "cargo install failed".into(),
            ));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        sanitize::validate_package_names(packages)?;

        let mut cmd = Command::new("cargo");
        cmd.arg("uninstall");

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "cargo uninstall".into(),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(
                "cargo uninstall failed".into(),
            ));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("cargo").is_ok()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // Cargo doesn't track reverse dependencies
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Use cargo search without limit (let search.rs handle limiting)
        let output = Command::new("cargo")
            .args(["search", query])
            .output()
            .map_err(|e| DeclarchError::Other(format!("cargo search failed: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();

        // Parse cargo search output:
        // name = "version"    # description
        for line in stdout.lines() {
            let line = line.trim();

            // Skip the summary line at the end
            if line.starts_with("... and") || line.starts_with("note:") {
                continue;
            }

            // Parse format: name = "version"    # description
            if let Some(eq_pos) = line.find(" = \"") {
                let name = line[..eq_pos].trim();
                let rest = &line[eq_pos + 4..];

                if let Some(quote_end) = rest.find('"') {
                    let version = &rest[..quote_end];

                    // Extract description after #
                    let description = if let Some(hash_pos) = rest.find('#') {
                        Some(rest[hash_pos + 1..].trim().to_string())
                    } else {
                        None
                    };

                    results.push(PackageSearchResult {
                        name: name.to_string(),
                        version: Some(version.to_string()),
                        description,
                        backend: Backend::Cargo,
                    });
                }
            }
        }

        Ok(results)
    }
}
