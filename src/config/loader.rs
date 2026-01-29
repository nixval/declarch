use crate::config::kdl::{ProjectMetadata, ConflictEntry, LifecycleConfig, PolicyConfig, parse_kdl_content};
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::utils::distro::DistroType;
use crate::utils::paths::expand_home;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct MergedConfig {
    pub packages: HashMap<PackageId, Vec<PathBuf>>,
    pub excludes: Vec<String>,
    /// Package mappings: config_name -> actual_package_name
    pub package_mappings: HashMap<String, String>,

    // === NEW: Additional config fields ===
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
    /// Returns: Vec of (package_name, Vec of backends)
    pub fn get_cross_backend_conflicts(&self) -> Vec<(String, Vec<Backend>)> {
        use std::collections::HashMap;

        let mut name_to_backends: HashMap<String, Vec<Backend>> = HashMap::new();

        // Group packages by name
        for pkg_id in self.packages.keys() {
            name_to_backends
                .entry(pkg_id.name.clone())
                .or_default()
                .push(pkg_id.backend.clone());
        }

        // Filter to only names with multiple backends
        name_to_backends
            .into_iter()
            .filter(|(_, backends)| backends.len() > 1)
            .collect()
    }
}

pub fn load_root_config(path: &Path) -> Result<MergedConfig> {
    let mut merged = MergedConfig::default();
    let mut visited_paths = std::collections::HashSet::new();

    recursive_load(path, &mut merged, &mut visited_paths)?;

    Ok(merged)
}

fn recursive_load(
    path: &Path,
    merged: &mut MergedConfig,
    visited: &mut std::collections::HashSet<PathBuf>,
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

    if visited.contains(&canonical_path) {
        return Ok(());
    }
    visited.insert(canonical_path.clone());

    let content = std::fs::read_to_string(&canonical_path)?;

    // Parsing KDL
    let raw = parse_kdl_content(&content)?;

    // Detect distro for conditional package processing
    let distro = DistroType::detect();

    // Process AUR packages (default, Arch-only)
    if distro.supports_aur() {
        for pkg_entry in raw.packages {
            let pkg_id = PackageId {
                name: pkg_entry.name,
                backend: Backend::Aur,
            };

            merged
                .packages
                .entry(pkg_id)
                .or_default()
                .push(canonical_path.clone());
        }
    }

    // Process Soar packages (cross-distro static binaries)
    for pkg_entry in raw.soar_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Soar,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process Flatpak packages (cross-distro)
    for pkg_entry in raw.flatpak_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Flatpak,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process npm packages (Node.js)
    for pkg_entry in raw.npm_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Npm,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process Yarn packages
    for pkg_entry in raw.yarn_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Yarn,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process pnpm packages
    for pkg_entry in raw.pnpm_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Pnpm,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process Bun packages
    for pkg_entry in raw.bun_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Bun,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process pip packages (Python)
    for pkg_entry in raw.pip_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Pip,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process Cargo packages (Rust)
    for pkg_entry in raw.cargo_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Cargo,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process Homebrew packages
    for pkg_entry in raw.brew_packages {
        let pkg_id = PackageId {
            name: pkg_entry.name,
            backend: Backend::Brew,
        };

        merged
            .packages
            .entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    // Process custom backend packages (user-defined)
    for (backend_name, packages) in raw.custom_packages {
        for pkg_entry in packages {
            let pkg_id = PackageId {
                name: pkg_entry.name,
                backend: Backend::Custom(backend_name.clone()),
            };

            merged
                .packages
                .entry(pkg_id)
                .or_default()
                .push(canonical_path.clone());
        }
    }

    merged.excludes.extend(raw.excludes);
    merged.package_mappings.extend(raw.package_mappings);

    // === NEW: Merge additional config fields ===

    // Meta: Only keep the first one (usually from root config)
    if merged.project_metadata.is_none()
        && raw.project_metadata.description.is_some()
        && raw.project_metadata.author.is_some()
        && raw.project_metadata.version.is_some()
    {
        merged.project_metadata = Some(raw.project_metadata);
    }

    // Conflicts: Accumulate from all configs
    merged.conflicts.extend(raw.conflicts);

    // Backend options: Merge (later configs override earlier ones)
    for (backend, opts) in raw.backend_options {
        merged
            .backend_options
            .entry(backend)
            .or_default()
            .extend(opts);
    }

    // Environment variables: Merge (later configs extend earlier ones)
    for (scope, vars) in raw.env {
        merged.env.entry(scope).or_default().extend(vars);
    }

    // Repositories: Merge (later configs extend earlier ones)
    for (backend, repos) in raw.package_sources {
        merged
            .package_sources
            .entry(backend)
            .or_default()
            .extend(repos);
    }

    // Policy: Last one wins
    if raw.policy.protected.iter().any(|p| !p.is_empty()) || raw.policy.orphans.is_some() {
        merged.policy = Some(raw.policy);
    }

    // Hooks: Merge (later configs extend earlier ones)
    if merged.lifecycle_actions.is_none() && !raw.lifecycle_actions.actions.is_empty() {
        merged.lifecycle_actions = Some(raw.lifecycle_actions);
    } else if let Some(ref mut merged_hooks) = merged.lifecycle_actions {
        if !raw.lifecycle_actions.actions.is_empty() {
            merged_hooks.actions.extend(raw.lifecycle_actions.actions);
        }
    }

    // Get parent directory safely - canonicalized paths should always have a parent
    // except for root paths, which is a case we should handle explicitly
    let parent_dir = canonical_path.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot determine parent directory for config file: {}",
            canonical_path.display()
        ))
    })?;

    for import_str in raw.imports {
        // Security: Validate import path to prevent path traversal attacks
        let import_path = if import_str.starts_with("~/") || import_str.starts_with("/") {
            // Absolute paths or home paths: allow but validate
            PathBuf::from(import_str)
        } else {
            // Relative paths: must be relative to current config directory
            // Security check: Block path traversal attempts
            if import_str.contains("..") {
                return Err(DeclarchError::ConfigError(format!(
                    "Path traversal blocked: import paths cannot contain '..'\n  Import: {}",
                    import_str
                )));
            }

            // Security check: Block absolute paths in relative import syntax
            if import_str.starts_with("/") {
                return Err(DeclarchError::ConfigError(format!(
                    "Absolute path not allowed in relative import\n  Import: {}",
                    import_str
                )));
            }

            parent_dir.join(import_str)
        };

        recursive_load(&import_path, merged, visited)?;
    }

    Ok(())
}
