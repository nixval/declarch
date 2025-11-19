use std::path::{Path, PathBuf};
use crate::config::kdl::parse_kdl_content;
use crate::utils::errors::{DeclarchError, Result};
use crate::utils::paths::expand_home;

#[derive(Debug, Default)]
pub struct MergedConfig {
    pub packages: Vec<String>,
    pub excludes: Vec<String>,
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
    visited: &mut std::collections::HashSet<PathBuf>
) -> Result<()> {
    let abs_path = expand_home(path)
        .map_err(|e| DeclarchError::Other(format!("Path expansion error: {}", e)))?;

    let canonical_path = std::fs::canonicalize(&abs_path)
        .map_err(|_e| DeclarchError::ConfigNotFound { path: abs_path.clone() })?;

    if visited.contains(&canonical_path) {
        return Ok(());
    }
    visited.insert(canonical_path.clone());

    let content = std::fs::read_to_string(&canonical_path)
        .map_err(|e| DeclarchError::FileReadError { path: canonical_path.clone(), reason: e.to_string() })?;
    
    let raw = parse_kdl_content(&content)?;

    merged.packages.extend(raw.packages);
    merged.excludes.extend(raw.excludes);

    let parent_dir = canonical_path.parent().unwrap();
    
    for import_str in raw.imports {
        let import_path = if import_str.starts_with("~/") || import_str.starts_with("/") {
            PathBuf::from(import_str)
        } else {
            parent_dir.join(import_str)
        };

        recursive_load(&import_path, merged, visited)?;
    }

    Ok(())
}
