mod selector_filter;

use crate::config::kdl::{
    ConflictEntry, LifecycleConfig, McpConfig, PolicyConfig, ProjectMetadata,
    parse_kdl_content_with_path,
};
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::utils::paths::expand_home;
use selector_filter::filter_content_by_selectors;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Track import chain for circular import detection
#[derive(Debug)]
struct ImportContext {
    /// Stack of files currently being loaded (for cycle detection)
    stack: Vec<PathBuf>,
    /// Set of all visited files
    visited: HashSet<PathBuf>,
}

impl ImportContext {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            visited: HashSet::new(),
        }
    }

    fn push(&mut self, path: PathBuf) -> Result<()> {
        // Check if this path is already in the stack (circular import)
        if let Some(pos) = self.stack.iter().position(|p| p == &path) {
            let cycle: Vec<String> = self.stack[pos..]
                .iter()
                .chain(std::iter::once(&path))
                .map(|p| p.display().to_string())
                .collect();
            return Err(DeclarchError::ConfigError(format!(
                "Circular import detected:\n  {}",
                cycle.join("\n  -> ")
            )));
        }

        self.stack.push(path.clone());
        self.visited.insert(path);
        Ok(())
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn contains(&self, path: &Path) -> bool {
        self.visited.contains(path)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoadSelectors {
    pub profile: Option<String>,
    pub host: Option<String>,
}

impl LoadSelectors {
    fn normalized(&self) -> Self {
        Self {
            profile: normalize_selector(&self.profile),
            host: normalize_selector(&self.host),
        }
    }

    fn is_empty(&self) -> bool {
        self.profile.is_none() && self.host.is_none()
    }
}

fn normalize_selector(selector: &Option<String>) -> Option<String> {
    selector
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

#[derive(Debug, Default)]
pub struct MergedConfig {
    /// All packages organized by PackageId (backend:name)
    pub packages: HashMap<PackageId, Vec<PathBuf>>,
    /// Packages to exclude from sync
    pub excludes: Vec<String>,
    /// Project metadata (merged from first config with meta)
    pub project_metadata: Option<ProjectMetadata>,
    /// Mutually exclusive packages (accumulated from all configs)
    pub conflicts: Vec<ConflictEntry>,
    /// Backend-specific configuration options (merged)
    pub backend_options: HashMap<String, HashMap<String, String>>,
    /// Environment variables for package operations (merged)
    pub env: HashMap<String, Vec<String>>,
    /// Custom package sources (merged)
    pub package_sources: HashMap<String, Vec<String>>,
    /// Package lifecycle policies (merged from last config)
    pub policy: Option<PolicyConfig>,
    /// Pre/post sync hooks (accumulated from all configs)
    pub lifecycle_actions: Option<LifecycleConfig>,
    /// Preferred editor from KDL config
    pub editor: Option<String>,
    /// Backend definitions loaded from imports
    pub backends: Vec<crate::backends::config::BackendConfig>,
    /// Source files for each backend definition in load order
    pub backend_sources: HashMap<String, Vec<PathBuf>>,
    /// Experimental feature flags merged from all config files
    pub experimental: HashSet<String>,
    /// MCP policy merged from configs
    pub mcp: Option<McpConfig>,
}

impl MergedConfig {
    pub fn get_unique_packages(&self) -> Vec<PackageId> {
        self.packages.keys().cloned().collect()
    }

    pub fn get_duplicates(&self) -> Vec<(&PackageId, &Vec<PathBuf>)> {
        self.packages
            .iter()
            .filter(|(_, sources)| sources.len() > 1)
            .collect()
    }

    /// Find packages with the same name across different backends
    pub fn get_cross_backend_conflicts(&self) -> Vec<(String, Vec<Backend>)> {
        let mut name_to_backends: HashMap<String, Vec<Backend>> = HashMap::new();

        for pkg_id in self.packages.keys() {
            name_to_backends
                .entry(pkg_id.name.clone())
                .or_default()
                .push(pkg_id.backend.clone());
        }

        name_to_backends
            .into_iter()
            .filter(|(_, backends)| backends.len() > 1)
            .collect()
    }

    /// Get all unique backends used in this configuration
    pub fn get_backends(&self) -> Vec<Backend> {
        let mut backends: Vec<Backend> = self
            .packages
            .keys()
            .map(|pkg_id| pkg_id.backend.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        backends.sort_by(|a, b| a.name().cmp(b.name()));
        backends
    }

    /// Check whether an experimental feature flag is enabled.
    pub fn is_experimental_enabled(&self, flag: &str) -> bool {
        self.experimental.contains(flag)
    }

    /// Get MCP mode, defaulting to read-only when unset.
    pub fn mcp_mode(&self) -> &str {
        self.mcp
            .as_ref()
            .and_then(|m| m.mode.as_deref())
            .unwrap_or("read-only")
    }

    /// Returns whether a write-capable MCP tool is explicitly allowed by config.
    pub fn is_mcp_tool_allowed(&self, tool_name: &str) -> bool {
        self.mcp
            .as_ref()
            .map(|m| m.allow_tools.iter().any(|t| t == tool_name))
            .unwrap_or(false)
    }
}

pub fn load_root_config(path: &Path) -> Result<MergedConfig> {
    load_root_config_with_selectors(path, &LoadSelectors::default())
}

pub fn load_root_config_with_selectors(
    path: &Path,
    selectors: &LoadSelectors,
) -> Result<MergedConfig> {
    let mut merged = MergedConfig::default();
    let mut context = ImportContext::new();
    let normalized = selectors.normalized();

    recursive_load(path, &mut merged, &mut context, &normalized)?;

    Ok(merged)
}

/// Filesystem-based configuration loader implementing the ConfigLoader trait
pub struct FilesystemConfigLoader;

impl crate::traits::ConfigLoader for FilesystemConfigLoader {
    fn load_root(&self, path: &Path) -> Result<MergedConfig> {
        load_root_config(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

fn recursive_load(
    path: &Path,
    merged: &mut MergedConfig,
    context: &mut ImportContext,
    selectors: &LoadSelectors,
) -> Result<()> {
    let abs_path = expand_home(path)
        .map_err(|e| DeclarchError::Other(format!("Path expansion error: {}", e)))?;

    let path_with_ext = if !abs_path.exists() && abs_path.extension().is_none() {
        abs_path.with_extension("kdl")
    } else {
        abs_path.clone()
    };

    let canonical_path =
        std::fs::canonicalize(&path_with_ext).map_err(|_e| DeclarchError::ConfigNotFound {
            path: path_with_ext.clone(),
        })?;

    // Check for circular imports using the context
    if context.contains(&canonical_path) {
        // Already processed in a different branch, skip
        return Ok(());
    }

    // Add to context for cycle detection
    context.push(canonical_path.clone())?;

    let content = std::fs::read_to_string(&canonical_path)?;
    let filtered_content = filter_content_by_selectors(&content, selectors)?;
    let file_path_str = canonical_path.display().to_string();
    let raw = parse_kdl_content_with_path(&filtered_content, Some(&file_path_str))?;

    // Process packages from unified storage (packages_by_backend)
    for (backend_name, packages) in raw.packages_by_backend {
        for pkg_entry in packages {
            let pkg_id = PackageId {
                name: pkg_entry.name,
                backend: Backend::from(backend_name.clone()),
            };

            merged
                .packages
                .entry(pkg_id)
                .or_default()
                .push(canonical_path.clone());
        }
    }

    // Merge other configuration
    merged.excludes.extend(raw.excludes);

    // Meta: Only keep the first one
    if merged.project_metadata.is_none() {
        let has_description = raw.project_metadata.description.is_some();
        let has_author = raw.project_metadata.author.is_some();

        if has_description || has_author {
            merged.project_metadata = Some(raw.project_metadata);
        }
    }

    // Editor: Only keep the first one (root config has priority)
    if merged.editor.is_none() && raw.editor.is_some() {
        merged.editor = raw.editor;
    }

    // Conflicts: Accumulate from all configs
    merged.conflicts.extend(raw.conflicts);

    // Backend options: Merge
    for (backend, opts) in raw.backend_options {
        merged
            .backend_options
            .entry(backend)
            .or_default()
            .extend(opts);
    }

    // Environment variables: Merge
    for (scope, vars) in raw.env {
        merged.env.entry(scope).or_default().extend(vars);
    }

    // Repositories: Merge
    for (backend, repos) in raw.package_sources {
        merged
            .package_sources
            .entry(backend)
            .or_default()
            .extend(repos);
    }

    // Policy: Last one wins
    if raw.policy.protected.iter().any(|p| !p.is_empty())
        || raw.policy.orphans.is_some()
        || raw.policy.require_backend.is_some()
        || raw.policy.forbid_hooks.is_some()
        || raw.policy.on_duplicate.is_some()
        || raw.policy.on_conflict.is_some()
    {
        merged.policy = Some(raw.policy);
    }

    // Hooks: Merge
    if merged.lifecycle_actions.is_none() && !raw.lifecycle_actions.actions.is_empty() {
        merged.lifecycle_actions = Some(raw.lifecycle_actions);
    } else if let Some(ref mut merged_hooks) = merged.lifecycle_actions
        && !raw.lifecycle_actions.actions.is_empty()
    {
        merged_hooks.actions.extend(raw.lifecycle_actions.actions);
    }

    // Experimental flags: merge as a unique set
    merged.experimental.extend(raw.experimental);

    // MCP policy: mode is last one wins, allow_tools is merged unique.
    if raw.mcp.mode.is_some() || !raw.mcp.allow_tools.is_empty() {
        let mcp = merged.mcp.get_or_insert_with(McpConfig::default);
        if let Some(mode) = raw.mcp.mode {
            mcp.mode = Some(mode);
        }
        for tool in raw.mcp.allow_tools {
            if !mcp.allow_tools.iter().any(|t| t == &tool) {
                mcp.allow_tools.push(tool);
            }
        }
    }

    // Process backend imports (NEW: explicit backend loading)
    let parent_dir = canonical_path.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot determine parent directory for config file: {}",
            canonical_path.display()
        ))
    })?;

    for backend_import in raw.backend_imports {
        let backend_path = if backend_import.starts_with("~/") {
            expand_home(PathBuf::from(backend_import.clone()).as_path())?
        } else if backend_import.starts_with('/') {
            PathBuf::from(backend_import.clone())
        } else {
            // Relative to parent directory
            parent_dir.join(backend_import.clone())
        };

        if backend_path.exists() {
            match crate::backends::user_parser::load_user_backends(&backend_path) {
                Ok(backends) => {
                    for backend in backends {
                        // Check for duplicate backend names
                        if merged.backends.iter().any(|b| b.name == backend.name) {
                            eprintln!(
                                "Warning: Duplicate backend '{}' from '{}'",
                                backend.name,
                                backend_path.display()
                            );
                        }
                        merged
                            .backend_sources
                            .entry(backend.name.clone())
                            .or_default()
                            .push(backend_path.clone());
                        merged.backends.push(backend);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load backends from '{}': {}",
                        backend_path.display(),
                        e
                    );
                }
            }
        } else {
            return Err(DeclarchError::ConfigError(format!(
                "Backend import not found: '{}' (resolved to: {})",
                backend_import,
                backend_path.display()
            )));
        }
    }

    // Process regular imports (modules)
    for import_str in raw.imports {
        let import_path = if import_str.starts_with("~/") || import_str.starts_with("/") {
            PathBuf::from(import_str)
        } else {
            // Security check: Block path traversal attempts
            // Check for .. in path components (not just substring)
            let normalized = import_str.replace('\\', "/");
            if normalized.split('/').any(|part| part == "..") {
                return Err(DeclarchError::ConfigError(
                    "Path traversal blocked: import paths cannot contain '..'".to_string(),
                ));
            }

            // Additional check: ensure import doesn't start with /
            if import_str.starts_with('/') {
                return Err(DeclarchError::ConfigError(
                    "Invalid import path: absolute paths not allowed".to_string(),
                ));
            }

            parent_dir.join(import_str)
        };

        match recursive_load(&import_path, merged, context, selectors) {
            Ok(()) => {}
            Err(DeclarchError::ConfigNotFound { .. }) => {
                // Silently skip missing imports
            }
            Err(e) => {
                context.pop();
                return Err(e);
            }
        }
    }

    // Remove from stack when done processing
    context.pop();

    Ok(())
}

#[cfg(test)]
mod tests;
