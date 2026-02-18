use super::GenericManager;
use crate::error::{DeclarchError, Result};
use crate::packages::traits::PackageSearchResult;
use crate::utils::regex_cache;
use serde_json::Value;

impl GenericManager {
    /// Parse search results based on configured format
    pub(super) fn parse_search_results(&self, stdout: &[u8]) -> Result<Vec<PackageSearchResult>> {
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
                    version: None,
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
    pub(super) fn parse_search_regex(&self, stdout: &str) -> Result<Vec<PackageSearchResult>> {
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
        let capture_name = |captures: &regex::Captures<'_>| {
            captures
                .get(name_group)
                .or_else(|| {
                    // Fallback: some regex patterns use alternatives where
                    // only one branch captures. Pick first non-empty group.
                    (1..captures.len()).find_map(|idx| captures.get(idx))
                })
                .map(|m| m.as_str().to_string())
        };

        // Multiline regex is evaluated against full stdout.
        // Non-multiline regex is evaluated line-by-line to avoid partial captures.
        if regex_str.contains("(?m)") {
            for captures in regex.captures_iter(stdout) {
                let name = capture_name(&captures).ok_or_else(|| {
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
        } else {
            for line in stdout.lines() {
                if let Some(captures) = regex.captures(line) {
                    let name = capture_name(&captures).ok_or_else(|| {
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
    pub(super) fn parse_local_search_results(
        &self,
        stdout: &[u8],
    ) -> Result<Vec<PackageSearchResult>> {
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
            .and(None)
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
