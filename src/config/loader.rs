use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::config::kdl::parse_kdl_content;
use crate::error::{DeclarchError, Result};
use crate::utils::paths::expand_home;
use crate::core::types::PackageId;


#[derive(Debug, Default)]
pub struct MergedConfig {
    pub packages: HashMap<PackageId, Vec<PathBuf>>,
    pub excludes: Vec<String>,
    /// Package aliases: config_name -> actual_package_name
    pub aliases: HashMap<String, String>,
}

impl MergedConfig {

    pub fn get_unique_packages(&self) -> Vec<PackageId> {
        self.packages.keys().cloned().collect()
    }


    pub fn get_duplicates(&self) -> Vec<(&PackageId, &Vec<PathBuf>)> {
        self.packages.iter()
            .filter(|(_, sources)| sources.len() > 1)
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
    visited: &mut std::collections::HashSet<PathBuf>
) -> Result<()> {
    let abs_path = expand_home(path)
        .map_err(|e| DeclarchError::Other(format!("Path expansion error: {}", e)))?;


    let path_with_ext = if !abs_path.exists() && abs_path.extension().is_none() {
        abs_path.with_extension("kdl")
    } else {
        abs_path.clone()
    };

    let canonical_path = std::fs::canonicalize(&path_with_ext)
        .map_err(|_e| DeclarchError::ConfigNotFound { path: path_with_ext.clone() })?;

    if visited.contains(&canonical_path) {
        return Ok(());
    }
    visited.insert(canonical_path.clone());

    let content = std::fs::read_to_string(&canonical_path)?;
    
    // Parsing KDL
    let raw = parse_kdl_content(&content)?;

  
    for pkg_str in raw.packages {
 
        let pkg_id: PackageId = pkg_str.parse().map_err(|e| DeclarchError::ConfigError(e))?;
        
        merged.packages.entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }

    merged.excludes.extend(raw.excludes);
    merged.aliases.extend(raw.aliases);

    // Get parent directory safely - canonicalized paths should always have a parent
    // except for root paths, which is a case we should handle explicitly
    let parent_dir = canonical_path
        .parent()
        .ok_or_else(|| DeclarchError::Other(
            format!("Cannot determine parent directory for config file: {}", canonical_path.display())
        ))?;

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
