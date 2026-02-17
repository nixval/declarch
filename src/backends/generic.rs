mod command_exec;
mod search_parsing;

use crate::backends::config::BackendConfig;
use crate::backends::parsers;
use crate::constants::BACKEND_COMMAND_TIMEOUT_SECS;
use crate::core::types::{Backend as CoreBackend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::ui;
use crate::utils::sanitize;
use command_exec::{run_command_with_timeout, run_interactive_command_with_timeout};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

/// Default timeout for backend commands (5 minutes)
const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(BACKEND_COMMAND_TIMEOUT_SECS);

/// Generic package manager that works with any backend configuration
pub struct GenericManager {
    config: BackendConfig,
    noconfirm: bool,
    backend_type: CoreBackend,
}

#[derive(Clone, Copy)]
enum CommandMode {
    ReadOnly,
    Mutating,
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
    /// Handles fallback if primary binary not available and fallback configured
    fn get_binary(&self) -> Result<String> {
        // Try primary binary first
        if let Some(bin) = self.config.binary.find_available() {
            return Ok(bin);
        }

        // If primary not available and fallback configured, try fallback
        if let Some(fallback_name) = &self.config.fallback {
            // Load fallback backend config
            let all_backends = crate::backends::load_all_backends_unified().map_err(|e| {
                DeclarchError::PackageManagerError(format!("Failed to load backend configs: {}", e))
            })?;

            if let Some(fallback_config) = all_backends.get(fallback_name)
                && let Some(fallback_bin) = fallback_config.binary.find_available()
            {
                return Ok(fallback_bin);
            }
        }

        // Neither primary nor fallback available
        Err(DeclarchError::PackageManagerError(format!(
            "{} not found. Please install {} first.",
            self.config.binary.primary(),
            self.config.name
        )))
    }

    /// Build command with optional sudo
    /// Uses the resolved binary (respecting fallback if needed)
    fn build_command(&self, cmd_str: &str, mode: CommandMode) -> Result<Command> {
        let binary = self.get_binary()?;

        // Replace common placeholders
        let cmd_str = self.replace_common_placeholders(cmd_str, &binary);

        let use_sudo = self.config.needs_sudo && matches!(mode, CommandMode::Mutating);
        let mut cmd = crate::utils::platform::build_shell_command(&cmd_str, use_sudo)?;

        // Apply configured environment variables to all backend commands.
        if let Some(env_vars) = &self.config.preinstall_env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        Ok(cmd)
    }

    /// Format package list for command
    ///
    /// SECURITY: Each package name is shell-escaped to prevent injection attacks.
    /// Even though packages are validated before calling this function, we add
    /// an extra layer of protection through proper escaping.
    fn format_packages(&self, packages: &[String]) -> String {
        packages
            .iter()
            .map(|p| sanitize::shell_escape(p))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format repository/source list for command template usage.
    /// Each source is shell-escaped to prevent injection.
    fn format_sources(&self) -> String {
        self.config
            .package_sources
            .as_ref()
            .map(|sources| {
                sources
                    .iter()
                    .map(|s| sanitize::shell_escape(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }

    /// Replace placeholders that are common across all command templates.
    fn replace_common_placeholders(&self, template: &str, binary: &str) -> String {
        template
            .replace("{binary}", binary)
            .replace("{repos}", &self.format_sources())
    }
}

impl PackageManager for GenericManager {
    fn backend_type(&self) -> CoreBackend {
        self.backend_type.clone()
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // Get list command or return error if not configured
        let list_cmd = self.config.list_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support listing installed packages (no list_cmd configured)",
                self.config.name
            ))
        })?;

        let cmd_str = list_cmd.clone();
        let mut cmd = self.build_command(&cmd_str, CommandMode::ReadOnly)?;

        let output = run_command_with_timeout(&mut cmd, DEFAULT_COMMAND_TIMEOUT).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            }
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

        let mut cmd = self.build_command(&cmd_str, CommandMode::Mutating)?;

        // Use interactive timeout function (5 minute timeout for install)
        let timeout = Duration::from_secs(300);
        let status = run_interactive_command_with_timeout(&mut cmd, timeout).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: format!("install: {}", cmd_str),
                reason: e.to_string(),
            }
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

        // Get remove command or return error if not configured
        let cmd_template = self.config.remove_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support removing packages (no remove_cmd configured)",
                self.config.name
            ))
        })?;

        // Security: Validate all package names before shell execution
        sanitize::validate_package_names(packages)?;
        let package_list = self.format_packages(packages);
        let mut cmd_str = cmd_template.replace("{packages}", &package_list);

        // Append noconfirm flag if configured and enabled
        if self.noconfirm
            && let Some(flag) = &self.config.noconfirm_flag
        {
            cmd_str.push(' ');
            cmd_str.push_str(flag);
        }

        let mut cmd = self.build_command(&cmd_str, CommandMode::Mutating)?;

        // Use interactive timeout function (5 minute timeout for remove)
        let timeout = Duration::from_secs(300);
        let status = run_interactive_command_with_timeout(&mut cmd, timeout).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: format!("remove: {}", cmd_str),
                reason: e.to_string(),
            }
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
        // Check primary binary
        if self.config.binary.find_available().is_some() {
            return true;
        }

        // Check fallback if configured
        if let Some(fallback_name) = &self.config.fallback
            && let Ok(all_backends) = crate::backends::load_all_backends_unified()
            && let Some(fallback_config) = all_backends.get(fallback_name)
        {
            return fallback_config.binary.find_available().is_some();
        }

        false
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
        // Security: Validate search query before shell execution
        sanitize::validate_search_query(query)?;

        let search_cmd = self.config.search_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support search",
                self.config.name
            ))
        })?;

        // Replace query placeholder; common placeholders are handled by build_command
        let cmd_str = search_cmd.replace("{query}", &sanitize::shell_escape(query));
        let mut cmd = self.build_command(&cmd_str, CommandMode::ReadOnly)?;

        // Use shorter timeout for search (30 seconds)
        let output = run_command_with_timeout(&mut cmd, Duration::from_secs(30)).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            }
        })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        // Parse search results using the configured format
        self.parse_search_results(&output.stdout)
    }

    fn supports_update(&self) -> bool {
        self.config.update_cmd.is_some()
    }

    fn update(&self) -> Result<()> {
        let update_cmd = self.config.update_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support update (no update_cmd configured)",
                self.config.name
            ))
        })?;

        let cmd_str = update_cmd.clone();
        let mut cmd = self.build_command(&cmd_str, CommandMode::Mutating)?;

        ui::info(&format!("Updating {} package index...", self.config.name));

        // Use standard timeout for update (2 minutes)
        let output = run_command_with_timeout(&mut cmd, Duration::from_secs(120)).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            }
        })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "Failed to update {} package index: {}",
                self.config.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn supports_cache_clean(&self) -> bool {
        self.config.cache_clean_cmd.is_some()
    }

    fn clean_cache(&self) -> Result<()> {
        let cache_clean_cmd = self.config.cache_clean_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support cache cleaning (no cache_clean_cmd configured)",
                self.config.name
            ))
        })?;

        let cmd_str = cache_clean_cmd.clone();
        let mut cmd = self.build_command(&cmd_str, CommandMode::Mutating)?;

        ui::info(&format!("Cleaning {} cache...", self.config.name));

        // Use standard timeout for cache clean (5 minutes - can be slow)
        let output = run_command_with_timeout(&mut cmd, Duration::from_secs(300)).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            }
        })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "Failed to clean {} cache: {}",
                self.config.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        ui::success(&format!("{} cache cleaned", self.config.name));
        Ok(())
    }

    fn supports_upgrade(&self) -> bool {
        self.config.upgrade_cmd.is_some()
    }

    fn upgrade(&self) -> Result<()> {
        let upgrade_cmd = self.config.upgrade_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support upgrade (no upgrade_cmd configured)",
                self.config.name
            ))
        })?;

        let cmd_str = upgrade_cmd.clone();
        let mut cmd = self.build_command(&cmd_str, CommandMode::Mutating)?;

        ui::info(&format!("Upgrading {} packages...", self.config.name));

        // Use longer timeout for upgrade (10 minutes - can be slow)
        let output = run_interactive_command_with_timeout(&mut cmd, Duration::from_secs(600))
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            })?;

        if !output.success() {
            return Err(DeclarchError::PackageManagerError(format!(
                "Failed to upgrade {} packages",
                self.config.name
            )));
        }

        ui::success(&format!("{} packages upgraded", self.config.name));
        Ok(())
    }

    fn supports_search_local(&self) -> bool {
        self.config.search_local_cmd.is_some()
    }

    fn search_local(&self, query: &str) -> Result<Vec<PackageSearchResult>> {
        // Security: Validate search query before shell execution
        sanitize::validate_search_query(query)?;

        let search_local_cmd = self.config.search_local_cmd.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(format!(
                "Backend '{}' does not support local search",
                self.config.name
            ))
        })?;

        // Replace query placeholder; common placeholders are handled by build_command
        let cmd_str = search_local_cmd.replace("{query}", &sanitize::shell_escape(query));
        let mut cmd = self.build_command(&cmd_str, CommandMode::ReadOnly)?;

        // Use shorter timeout for search (30 seconds)
        let output = run_command_with_timeout(&mut cmd, Duration::from_secs(30)).map_err(|e| {
            DeclarchError::SystemCommandFailed {
                command: cmd_str.clone(),
                reason: e.to_string(),
            }
        })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        // Parse search results using the configured format
        // For local search, we use list parsing format since it's typically simpler output
        self.parse_local_search_results(&output.stdout)
    }
}

#[cfg(test)]
mod tests;
