use crate::backends::config::BackendConfig;
use crate::backends::parsers;
use crate::core::types::{Backend as CoreBackend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::utils::sanitize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Generic package manager that works with any backend configuration
pub struct GenericManager {
    config: BackendConfig,
    noconfirm: bool,
    backend_type: CoreBackend,
}

impl GenericManager {
    /// Create a new generic manager from configuration
    pub fn from_config(
        mut config: BackendConfig,
        backend_type: CoreBackend,
        noconfirm: bool,
    ) -> Self {
        // Set default columns if not specified
        if config.list_name_col.is_none() {
            config.list_name_col = Some(0);
        }
        if config.list_version_col.is_none() {
            config.list_version_col = Some(1);
        }

        Self {
            config,
            noconfirm,
            backend_type,
        }
    }

    /// Get the actual binary to use (first available from alternatives)
    fn get_binary(&self) -> Result<String> {
        self.config.binary.find_available().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "{} not found. Please install {} first.",
                self.config.binary.primary(),
                self.config.name
            ))
        })
    }

    /// Build command with optional sudo
    fn build_command(&self, cmd_str: &str) -> Result<Command> {
        let _binary = self.get_binary()?;

        // Replace {packages} placeholder if present (for install/remove)
        let cmd_str = cmd_str.to_string();

        if self.config.needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg("sh").arg("-c").arg(cmd_str);
            Ok(cmd)
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(cmd_str);
            Ok(cmd)
        }
    }

    /// Format package list for command
    fn format_packages(&self, packages: &[String]) -> String {
        packages.join(" ")
    }
}

impl PackageManager for GenericManager {
    fn backend_type(&self) -> CoreBackend {
        self.backend_type.clone()
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.config.list_cmd)
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: self.config.list_cmd.clone(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "Failed to list packages for {}: {}",
                self.config.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        parsers::parse_package_list(&output.stdout, &self.config)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        // Security: Validate all package names before shell execution
        sanitize::validate_package_names(packages)?;

        let cmd_template = &self.config.install_cmd;
        let package_list = self.format_packages(packages);
        let mut cmd_str = cmd_template.replace("{packages}", &package_list);

        // Append noconfirm flag if configured and enabled
        if self.noconfirm
            && let Some(flag) = &self.config.noconfirm_flag
        {
            cmd_str.push(' ');
            cmd_str.push_str(flag);
        }

        let mut cmd = self.build_command(&cmd_str)?;

        // Set environment variables if configured
        if let Some(env_vars) = &self.config.preinstall_env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: format!("install: {}", cmd_str),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "{} install failed",
                self.config.name
            )));
        }

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        // Security: Validate all package names before shell execution
        sanitize::validate_package_names(packages)?;

        let cmd_template = &self.config.remove_cmd;
        let package_list = self.format_packages(packages);
        let mut cmd_str = cmd_template.replace("{packages}", &package_list);

        // Append noconfirm flag if configured and enabled
        if self.noconfirm
            && let Some(flag) = &self.config.noconfirm_flag
        {
            cmd_str.push(' ');
            cmd_str.push_str(flag);
        }

        let mut cmd = self.build_command(&cmd_str)?;

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: format!("remove: {}", cmd_str),
                reason: e.to_string(),
            })?;

        if !status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "{} remove failed",
                self.config.name
            )));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        self.config.binary.find_available().is_some()
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        // For generic backends, we can't determine dependencies without backend-specific logic
        // Return empty list as default (safe behavior)
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        self.config.search_cmd.is_some()
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        let search_cmd = self.config.search_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support search",
                self.config.name
            ))
        })?;

        // Replace {query} placeholder
        let cmd_str = search_cmd.replace("{query}", query);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        // Parse search results using the configured format
        self.parse_search_results(&output.stdout)
    }
}

impl GenericManager {
    /// Parse search results based on configured format
    fn parse_search_results(&self, stdout: &[u8]) -> Result<Vec<PackageSearchResult>> {
        let format = self.config.search_format.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Search format not configured for backend '{}'",
                self.config.name
            ))
        })?;

        let stdout_str = String::from_utf8_lossy(stdout);

        match format {
            crate::backends::config::OutputFormat::Json => self.parse_search_json(&stdout_str),
            crate::backends::config::OutputFormat::SplitWhitespace => {
                self.parse_search_whitespace(&stdout_str)
            }
            crate::backends::config::OutputFormat::TabSeparated => {
                self.parse_search_tab(&stdout_str)
            }
            crate::backends::config::OutputFormat::Regex => self.parse_search_regex(&stdout_str),
            crate::backends::config::OutputFormat::Custom => {
                // Custom format - not supported for search
                Ok(Vec::new())
            }
        }
    }

    /// Parse JSON search results
    fn parse_search_json(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let json_path = self.config.search_json_path.as_deref().unwrap_or("");
        let name_key = self.config.search_name_key.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(
                "search_name_key not configured for JSON search".into(),
            )
        })?;

        // Get the results array
        let results_value = if json_path.is_empty() {
            serde_json::from_str::<serde_json::Value>(stdout)?
        } else {
            // Navigate JSON path (simple implementation for nested paths)
            let value: serde_json::Value = serde_json::from_str(stdout)?;
            self.navigate_json_path(&value, json_path)?
        };

        let results_array = results_value.as_array().ok_or_else(|| {
            DeclarchError::PackageManagerError("Search results is not a JSON array".into())
        })?;

        let version_key = self.config.search_version_key.as_deref();
        let desc_key = self.config.search_desc_key.as_deref();

        let mut results = Vec::new();
        for item in results_array {
            if let Some(obj) = item.as_object() {
                let name = obj.get(name_key).and_then(|v| v.as_str()).ok_or_else(|| {
                    DeclarchError::PackageManagerError(format!(
                        "Missing or invalid 'name' field in search result"
                    ))
                })?;

                let version = version_key
                    .and_then(|key| obj.get(key))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let description = desc_key
                    .and_then(|key| obj.get(key))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                results.push(PackageSearchResult {
                    name: name.to_string(),
                    version,
                    description,
                    backend: self.backend_type.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Navigate JSON path (simple implementation)
    fn navigate_json_path(
        &self,
        value: &serde_json::Value,
        path: &str,
    ) -> Result<serde_json::Value> {
        let mut current = value;
        for key in path.split('.') {
            current = current.get(key).ok_or_else(|| {
                DeclarchError::PackageManagerError(format!(
                    "JSON path '{}' not found in search results",
                    path
                ))
            })?;
        }
        Ok(current.clone())
    }

    /// Parse whitespace-separated search results
    fn parse_search_whitespace(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_col = self.config.search_name_col.ok_or_else(|| {
            DeclarchError::PackageManagerError("search_name_col not configured".into())
        })?;

        let desc_col = self.config.search_desc_col.unwrap_or(1);

        let mut results = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > name_col {
                let name = parts[name_col].to_string();
                let description = if parts.len() > desc_col {
                    Some(parts[desc_col].to_string())
                } else {
                    None
                };

                results.push(PackageSearchResult {
                    name,
                    version: None, // Search results typically don't have version
                    description,
                    backend: self.backend_type.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Parse tab-separated search results
    fn parse_search_tab(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_col = self.config.search_name_col.ok_or_else(|| {
            DeclarchError::PackageManagerError("search_name_col not configured".into())
        })?;

        let desc_col = self.config.search_desc_col.unwrap_or(1);

        let mut results = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() > name_col {
                let name = parts[name_col].to_string();
                let description = if parts.len() > desc_col && !parts[desc_col].is_empty() {
                    Some(parts[desc_col].to_string())
                } else {
                    None
                };

                results.push(PackageSearchResult {
                    name,
                    version: None,
                    description,
                    backend: self.backend_type.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Parse regex-based search results
    fn parse_search_regex(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let regex_str = self.config.search_regex.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError("search_regex not configured".into())
        })?;

        let name_group = self.config.search_regex_name_group.ok_or_else(|| {
            DeclarchError::PackageManagerError("search_regex_name_group not configured".into())
        })?;

        let desc_group = self.config.search_regex_desc_group.unwrap_or(1);

        let regex = regex::Regex::new(regex_str).map_err(|e| {
            DeclarchError::PackageManagerError(format!("Invalid search regex: {}", e))
        })?;

        let mut results = Vec::new();
        for line in stdout.lines() {
            if let Some(captures) = regex.captures(line) {
                let name = captures
                    .get(name_group)
                    .map(|m| m.as_str().to_string())
                    .ok_or_else(|| {
                        DeclarchError::PackageManagerError(
                            "Regex name group didn't capture anything".into(),
                        )
                    })?;

                let description = captures.get(desc_group).map(|m| m.as_str().to_string());

                results.push(PackageSearchResult {
                    name,
                    version: None,
                    description,
                    backend: self.backend_type.clone(),
                });
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::{BackendConfig, BinarySpecifier};
    use crate::core::types::Backend;

    #[test]
    fn test_format_packages() {
        let config = BackendConfig {
            name: "test".to_string(),
            binary: BinarySpecifier::Single("echo".to_string()),
            ..Default::default()
        };

        let manager = GenericManager::from_config(config, Backend::Aur, false);
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];

        assert_eq!(manager.format_packages(&packages), "pkg1 pkg2");
    }

    #[test]
    fn test_binary_specifier_primary() {
        let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
        assert_eq!(bin.primary(), "paru");
    }
}
