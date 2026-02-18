mod import_context;
mod merging;
mod path_resolution;
mod selector_filter;

use crate::config::kdl::{
    ConflictEntry, LifecycleConfig, McpConfig, PolicyConfig, ProjectMetadata, RawConfig,
    parse_kdl_content_with_path,
};
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use import_context::ImportContext;
use merging::merge_raw_config;
use path_resolution::{
    parent_dir_of, resolve_backend_import_path, resolve_module_import_path,
    resolve_primary_config_path,
};
use selector_filter::filter_content_by_selectors;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    let canonical_path = resolve_primary_config_path(path)?;

    // Check for circular imports using the context
    if context.contains(&canonical_path) {
        // Already processed in a different branch, skip
        return Ok(());
    }

    // Add to context for cycle detection
    context.push(canonical_path.clone())?;

    let raw = load_raw_config(&canonical_path, selectors)?;
    let pending_imports = merge_raw_config(merged, raw, &canonical_path);

    // Process backend imports (NEW: explicit backend loading)
    let parent_dir = parent_dir_of(&canonical_path)?;

    for backend_import in pending_imports.backend_imports {
        process_backend_import(merged, parent_dir.as_path(), &backend_import)?;
    }

    // Process regular imports (modules)
    for import_str in pending_imports.imports {
        let import_path = resolve_module_import_path(parent_dir.as_path(), &import_str)?;

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

fn load_raw_config(path: &Path, selectors: &LoadSelectors) -> Result<RawConfig> {
    let content = std::fs::read_to_string(path)?;
    let filtered_content = filter_content_by_selectors(&content, selectors)?;
    let file_path_str = path.display().to_string();
    parse_kdl_content_with_path(&filtered_content, Some(&file_path_str))
}

fn process_backend_import(
    merged: &mut MergedConfig,
    parent_dir: &Path,
    backend_import: &str,
) -> Result<()> {
    let backend_path = resolve_backend_import_path(parent_dir, backend_import)?;

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
        return Ok(());
    }

    Err(DeclarchError::ConfigError(format!(
        "Backend import not found: '{}' (resolved to: {})",
        backend_import,
        backend_path.display()
    )))
}

#[cfg(test)]
mod tests;
