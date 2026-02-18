use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::core::types::Backend;
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use std::path::{Path, PathBuf};

pub(super) fn parse_query(query: &str) -> (Option<Backend>, String) {
    if let Some((backend, name)) = query.split_once(':') {
        let backend = backend.trim();
        let name = name.trim();
        if !backend.is_empty() && !name.is_empty() {
            return (Some(Backend::from(backend)), name.to_string());
        }
    }

    (None, query.trim().to_string())
}

pub(super) fn looks_like_module_query(query: &str) -> bool {
    query.contains('/') || query.ends_with(".kdl")
}

pub(super) fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
    selectors: &LoadSelectors,
) -> Result<MergedConfig> {
    let mut merged = loader::load_root_config_with_selectors(config_path, selectors)?;

    for module_name in extra_modules {
        let final_path = resolve_module_path(module_name)?;
        output::info(&format!("  Loading module: {}", final_path.display()));

        let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}

pub(super) fn resolve_module_path(module_name: &str) -> Result<PathBuf> {
    let module_path = paths::module_file(module_name);

    if let Ok(path) = module_path
        && path.exists()
    {
        return Ok(path);
    }

    let direct = PathBuf::from(module_name);
    if direct.exists() {
        return Ok(direct);
    }

    Err(DeclarchError::Other(format!(
        "Module not found: {}",
        module_name
    )))
}
