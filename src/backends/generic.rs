use crate::backends::config::BackendConfig;
use crate::backends::parsers;
use crate::core::types::{Backend as CoreBackend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::ui;
use crate::utils::regex_cache;
use crate::utils::sanitize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Default timeout for backend commands (5 minutes)
const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(300);

/// Execute a command with timeout (non-interactive)
fn run_command_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<std::process::Output> {
    let cmd_debug = format!("{:?}", cmd);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: e.to_string(),
        })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: "Failed to capture stdout".to_string(),
        })?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: "Failed to capture stderr".to_string(),
        })?;

    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::BufReader::new(stdout).read_to_end(&mut buf);
        buf
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::BufReader::new(stderr).read_to_end(&mut buf);
        buf
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();
                    return Err(DeclarchError::SystemCommandFailed {
                        command: cmd_debug,
                        reason: format!("Command timed out after {} seconds", timeout.as_secs()),
                    });
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(DeclarchError::SystemCommandFailed {
                    command: cmd_debug,
                    reason: e.to_string(),
                });
            }
        }
    };

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();

    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

/// Execute an interactive command with timeout (shows real-time output)
fn run_interactive_command_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> Result<std::process::ExitStatus> {
    let cmd_debug = format!("{:?}", cmd);

    // Set up interactive stdio
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: cmd_debug.clone(),
            reason: e.to_string(),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if start.elapsed() > timeout {
                    ui::warning(&format!(
                        "Command timed out after {} seconds",
                        timeout.as_secs()
                    ));
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DeclarchError::SystemCommandFailed {
                        command: cmd_debug,
                        reason: format!("Command timed out after {} seconds", timeout.as_secs()),
                    });
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(DeclarchError::SystemCommandFailed {
                    command: cmd_debug,
                    reason: e.to_string(),
                });
            }
        }
    }
}

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
        let mut cmd = if use_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg("sh").arg("-c").arg(cmd_str);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(cmd_str);
            cmd
        };

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
            crate::backends::config::OutputFormat::JsonLines => {
                self.parse_search_json_lines(&stdout_str)
            }
            crate::backends::config::OutputFormat::NpmJson => {
                self.parse_search_npm_json(&stdout_str)
            }
            crate::backends::config::OutputFormat::JsonObjectKeys => {
                // JsonObjectKeys is for list operations only (object keys as package names)
                Err(DeclarchError::PackageManagerError(
                    "JsonObjectKeys format is not supported for search operations".into(),
                ))
            }
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
                    DeclarchError::PackageManagerError(
                        "Missing or invalid 'name' field in search result".to_string(),
                    )
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

    /// Parse JSON Lines (NDJSON) search results
    /// Each line is a separate JSON object
    fn parse_search_json_lines(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_key = self.config.search_name_key.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(
                "search_name_key not configured for JSON Lines search".into(),
            )
        })?;

        let version_key = self.config.search_version_key.as_deref();
        let desc_key = self.config.search_desc_key.as_deref();
        let mut results = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Try to parse each line as JSON
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    if let Some(Value::String(name)) = json.get(name_key) {
                        let version = version_key
                            .and_then(|key| json.get(key))
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        let description = desc_key
                            .and_then(|key| json.get(key))
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
                Err(_) => {
                    // Skip lines that aren't valid JSON
                    continue;
                }
            }
        }

        Ok(results)
    }

    /// Parse NPM-style JSON search results
    /// Format: [\n{...}\n,\n{...}\n]
    fn parse_search_npm_json(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_key = self.config.search_name_key.as_ref().ok_or_else(|| {
            DeclarchError::PackageManagerError(
                "search_name_key not configured for NPM JSON search".into(),
            )
        })?;

        let version_key = self.config.search_version_key.as_deref();
        let desc_key = self.config.search_desc_key.as_deref();
        let mut results = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();

            // Skip array markers and commas
            if line.is_empty() || line == "[" || line == "]" || line == "," {
                continue;
            }

            // Lines might end with comma, remove it
            let line = line.trim_end_matches(',');

            // Try to parse as JSON object
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    if let Some(Value::String(name)) = json.get(name_key) {
                        let version = version_key
                            .and_then(|key| json.get(key))
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        let description = desc_key
                            .and_then(|key| json.get(key))
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
                Err(_) => {
                    // Skip non-JSON lines
                    continue;
                }
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

        let regex = regex_cache::get_cached_regex(regex_str).map_err(|e| {
            DeclarchError::PackageManagerError(format!("Invalid search regex: {}", e))
        })?;

        let mut results = Vec::new();

        // Try matching against entire stdout first (for multiline patterns)
        for captures in regex.captures_iter(stdout) {
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

        // If no matches and pattern doesn't have multiline flag, try line-by-line
        if results.is_empty() && !regex_str.contains("(?m)") {
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
        }

        Ok(results)
    }

    /// Parse local search results using search_local format configuration
    fn parse_local_search_results(&self, stdout: &[u8]) -> Result<Vec<PackageSearchResult>> {
        let stdout_str = String::from_utf8_lossy(stdout);

        // Use search_local format if configured, otherwise fall back to list_format
        let format = self
            .config
            .search_local_format
            .as_ref()
            .unwrap_or(&self.config.list_format);

        match format {
            crate::backends::config::OutputFormat::Json => {
                self.parse_local_search_json(&stdout_str)
            }
            crate::backends::config::OutputFormat::SplitWhitespace => {
                self.parse_local_search_whitespace(&stdout_str)
            }
            crate::backends::config::OutputFormat::TabSeparated => {
                self.parse_local_search_tab(&stdout_str)
            }
            crate::backends::config::OutputFormat::Regex => {
                self.parse_local_search_regex(&stdout_str)
            }
            _ => {
                // For other formats, just use line-by-line parsing
                let mut results = Vec::new();
                for line in stdout_str.lines() {
                    let name = line.trim().to_string();
                    if !name.is_empty() {
                        results.push(PackageSearchResult {
                            name,
                            version: None,
                            description: None,
                            backend: self.backend_type.clone(),
                        });
                    }
                }
                Ok(results)
            }
        }
    }

    /// Parse whitespace-separated local search results
    fn parse_local_search_whitespace(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_col = self
            .config
            .search_local_name_col
            .or(self.config.list_name_col)
            .unwrap_or(0);
        let version_col = self
            .config
            .search_local_version_key
            .as_ref()
            .and_then(|_| None) // version_key is for JSON, use version_col from list for whitespace
            .or(self.config.list_version_col);

        let mut results = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > name_col {
                let name = parts[name_col].to_string();
                let version = version_col.and_then(|col| parts.get(col).map(|&v| v.to_string()));

                if !name.is_empty() {
                    results.push(PackageSearchResult {
                        name,
                        version,
                        description: None,
                        backend: self.backend_type.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Parse tab-separated local search results
    fn parse_local_search_tab(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let name_col = self
            .config
            .search_local_name_col
            .or(self.config.list_name_col)
            .unwrap_or(0);

        let mut results = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() > name_col {
                let name = parts[name_col].to_string();
                if !name.is_empty() {
                    results.push(PackageSearchResult {
                        name,
                        version: None,
                        description: None,
                        backend: self.backend_type.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Parse JSON local search results
    fn parse_local_search_json(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let json_path = self
            .config
            .search_local_json_path
            .as_deref()
            .or(self.config.list_json_path.as_deref())
            .unwrap_or("");
        let name_key = self
            .config
            .search_local_name_key
            .as_ref()
            .or(self.config.list_name_key.as_ref())
            .ok_or_else(|| {
                DeclarchError::PackageManagerError(
                    "search_local_name_key not configured for JSON search".into(),
                )
            })?;

        // Get the results array
        let results_value = if json_path.is_empty() {
            serde_json::from_str::<serde_json::Value>(stdout)?
        } else {
            let value: serde_json::Value = serde_json::from_str(stdout)?;
            self.navigate_json_path(&value, json_path)?
        };

        let results_array = results_value.as_array().ok_or_else(|| {
            DeclarchError::PackageManagerError("Local search results is not a JSON array".into())
        })?;

        let version_key = self
            .config
            .search_local_version_key
            .as_deref()
            .or(self.config.list_version_key.as_deref());

        let mut results = Vec::new();
        for item in results_array {
            if let Some(obj) = item.as_object() {
                let name = obj.get(name_key).and_then(|v| v.as_str()).ok_or_else(|| {
                    DeclarchError::PackageManagerError(
                        "Missing or invalid 'name' field in local search result".to_string(),
                    )
                })?;

                let version = version_key
                    .and_then(|key| obj.get(key))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                results.push(PackageSearchResult {
                    name: name.to_string(),
                    version,
                    description: None,
                    backend: self.backend_type.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Parse regex-based local search results
    fn parse_local_search_regex(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
        let regex_str = self
            .config
            .search_local_regex
            .as_ref()
            .or(self.config.list_regex.as_ref())
            .ok_or_else(|| {
                DeclarchError::PackageManagerError(
                    "search_local_regex not configured for regex format".into(),
                )
            })?;

        let name_group = self
            .config
            .search_local_regex_name_group
            .or(self.config.list_regex_name_group)
            .unwrap_or(1);

        let regex = regex_cache::get_cached_regex(regex_str).map_err(|e| {
            DeclarchError::PackageManagerError(format!("Invalid local search regex: {}", e))
        })?;

        let mut results = Vec::new();

        for line in stdout.lines() {
            if let Some(captures) = regex.captures(line) {
                let name = captures
                    .get(name_group)
                    .map(|m: regex::Match<'_>| m.as_str().to_string())
                    .ok_or_else(|| {
                        DeclarchError::PackageManagerError(
                            "Regex name group didn't capture anything".into(),
                        )
                    })?;

                results.push(PackageSearchResult {
                    name,
                    version: None,
                    description: None,
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
            fallback: None,
            ..Default::default()
        };

        let manager = GenericManager::from_config(config, Backend::from("aur"), false);
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];

        assert_eq!(manager.format_packages(&packages), "pkg1 pkg2");
    }

    #[test]
    fn test_binary_specifier_primary() {
        let bin = BinarySpecifier::Multiple(vec!["paru".to_string(), "yay".to_string()]);
        assert_eq!(bin.primary(), "paru");
    }

    #[test]
    fn test_sudo_applies_only_to_mutating_operations() {
        let config = BackendConfig {
            name: "test".to_string(),
            binary: BinarySpecifier::Single("sh".to_string()),
            needs_sudo: true,
            ..Default::default()
        };
        let manager = GenericManager::from_config(config, Backend::from("aur"), false);

        let read_cmd = manager
            .build_command("{binary} -c 'echo read'", CommandMode::ReadOnly)
            .expect("read command should build");
        let read_debug = format!("{:?}", read_cmd);
        assert!(
            !read_debug.contains("\"sudo\""),
            "read-only commands must not use sudo: {}",
            read_debug
        );

        let write_cmd = manager
            .build_command("{binary} -c 'echo write'", CommandMode::Mutating)
            .expect("write command should build");
        let write_debug = format!("{:?}", write_cmd);
        assert!(
            write_debug.contains("\"sudo\""),
            "mutating commands should use sudo when needs_sudo=true: {}",
            write_debug
        );
    }
}
