use crate::config::kdl::parse_kdl_content_with_path;
use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn resolve_import_path(base_file: &Path, import: &str) -> Result<PathBuf> {
    let parent = base_file.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot determine parent directory for {}",
            base_file.display()
        ))
    })?;

    let candidate = if import.starts_with("~/") {
        crate::utils::paths::expand_home(Path::new(import))?
    } else if import.starts_with('/') {
        PathBuf::from(import)
    } else {
        parent.join(import)
    };

    if candidate.exists() || candidate.extension().is_some() {
        Ok(candidate)
    } else {
        Ok(candidate.with_extension("kdl"))
    }
}

pub(super) fn collect_lint_files(config_path: &Path, modules: &[String]) -> Result<Vec<PathBuf>> {
    let mut files: BTreeSet<PathBuf> = BTreeSet::new();
    discover_lint_files_recursive(config_path, &mut files)?;

    for module in modules {
        let module_path = resolve_module_path(module)?;
        discover_lint_files_recursive(&module_path, &mut files)?;
    }

    Ok(files.into_iter().collect())
}

pub(super) fn discover_lint_files_recursive(
    path: &Path,
    files: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !files.insert(canonical.clone()) {
        return Ok(());
    }

    if !canonical.exists() {
        return Ok(());
    }

    let content = match fs::read_to_string(&canonical) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };

    let file_str = canonical.display().to_string();
    let raw = match parse_kdl_content_with_path(&content, Some(&file_str)) {
        Ok(raw) => raw,
        Err(_) => return Ok(()),
    };

    for import in raw.imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            continue;
        }

        let resolved = resolve_import_path(&canonical, &import)?;
        if resolved.exists() {
            discover_lint_files_recursive(&resolved, files)?;
        }
    }

    for import in raw.backend_imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            continue;
        }

        let resolved = resolve_import_path(&canonical, &import)?;
        if resolved.exists() {
            discover_lint_files_recursive(&resolved, files)?;
        }
    }

    Ok(())
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

fn resolve_module_path(module_name: &str) -> Result<PathBuf> {
    if let Ok(path) = paths::module_file(module_name)
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
