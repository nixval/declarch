//! Module path resolution utilities
//!
//! Provides functions for resolving module paths in the declarch configuration.

use crate::constants::CONFIG_EXTENSION;
use crate::error::{DeclarchError, Result};
use crate::utils::paths;
use std::path::PathBuf;

/// Resolve module path from module name
///
/// Takes an optional module name and returns the resolved path.
/// - `Some(name)` → `modules/{name}.kdl`
/// - `None` → `modules/others.kdl` (fallback)
///
/// Supports nested module paths using "/" separator.
///
/// # Examples
///
/// ```rust,ignore
/// // Returns modules/others.kdl
/// resolve_module_path(None)?;
///
/// // Returns modules/base.kdl
/// resolve_module_path(Some("base"))?;
///
/// // Returns modules/linux/notes.kdl
/// resolve_module_path(Some("linux/notes"))?;
/// ```
pub fn resolve_module_path(module: Option<&str>) -> Result<PathBuf> {
    let modules_dir = paths::modules_dir()?;

    let target_file = match module {
        Some(mod_name) => {
            // Split on "/" to handle nested paths
            let parts: Vec<&str> = mod_name.split('/').collect();

            let file_name = format!(
                "{}.{}",
                parts
                    .last()
                    .ok_or_else(|| DeclarchError::Other("Invalid module path".to_string()))?,
                CONFIG_EXTENSION
            );
            let dir_path = parts
                .iter()
                .take(parts.len() - 1)
                .fold(modules_dir, |acc, part| acc.join(part));

            dir_path.join(file_name)
        }
        None => modules_dir.join(format!("others.{}", CONFIG_EXTENSION)),
    };

    Ok(target_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_module_path_none() {
        let path = resolve_module_path(None);
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("others.kdl"));
    }

    #[test]
    fn test_resolve_module_path_simple() {
        let path = resolve_module_path(Some("base"));
        assert!(path.is_ok());
        let path = path.unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(file_name, "base.kdl");
    }

    #[test]
    fn test_resolve_module_path_nested() {
        let path = resolve_module_path(Some("linux/notes"));
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("linux"));
        let file_name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(file_name, "notes.kdl");
    }
}
