use crate::error::{DeclarchError, Result};
use crate::utils::paths::expand_home;
use std::path::{Path, PathBuf};

pub(super) fn resolve_primary_config_path(path: &Path) -> Result<PathBuf> {
    let abs_path = expand_home(path)
        .map_err(|e| DeclarchError::Other(format!("Path expansion error: {}", e)))?;

    let path_with_ext = if !abs_path.exists() && abs_path.extension().is_none() {
        abs_path.with_extension("kdl")
    } else {
        abs_path
    };

    std::fs::canonicalize(&path_with_ext).map_err(|_e| DeclarchError::ConfigNotFound {
        path: path_with_ext,
    })
}

pub(super) fn parent_dir_of(path: &Path) -> Result<PathBuf> {
    path.parent().map(Path::to_path_buf).ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot determine parent directory for config file: {}",
            path.display()
        ))
    })
}

pub(super) fn resolve_backend_import_path(
    parent_dir: &Path,
    backend_import: &str,
) -> Result<PathBuf> {
    if backend_import.starts_with("~/") {
        return expand_home(Path::new(backend_import))
            .map_err(|e| DeclarchError::Other(format!("Path expansion error: {}", e)));
    }

    if backend_import.starts_with('/') {
        return Ok(PathBuf::from(backend_import));
    }

    Ok(parent_dir.join(backend_import))
}

pub(super) fn resolve_module_import_path(parent_dir: &Path, import_str: &str) -> Result<PathBuf> {
    if import_str.starts_with("~/") || import_str.starts_with('/') {
        return Ok(PathBuf::from(import_str));
    }

    let normalized = import_str.replace('\\', "/");
    if normalized.split('/').any(|part| part == "..") {
        return Err(DeclarchError::ConfigError(
            "Path traversal blocked: import paths cannot contain '..'".to_string(),
        ));
    }

    Ok(parent_dir.join(import_str))
}

#[cfg(test)]
mod tests {
    use super::{resolve_module_import_path, resolve_primary_config_path};
    use tempfile::tempdir;

    #[test]
    fn module_import_blocks_traversal() {
        let root = tempdir().expect("tempdir should be created");
        let err = resolve_module_import_path(root.path(), "../outside.kdl")
            .expect_err("traversal should fail");
        assert!(err.to_string().contains("Path traversal blocked"));
    }

    #[test]
    fn primary_path_supports_missing_extension_lookup() {
        let root = tempdir().expect("tempdir should be created");
        let config_path = root.path().join("declarch.kdl");
        std::fs::write(&config_path, "pkg { aur { git } }").expect("fixture write should succeed");

        let resolved = resolve_primary_config_path(&root.path().join("declarch"))
            .expect("path should resolve with implicit .kdl");
        assert_eq!(
            resolved,
            config_path.canonicalize().expect("canonicalize fixture")
        );
    }
}
