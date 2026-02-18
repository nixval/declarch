use crate::error::{DeclarchError, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn backup_kdl_file(file_path: &Path) -> Result<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = file_path.with_extension(format!("kdl.bak.{}", timestamp));

    fs::copy(file_path, &backup_path).map_err(|e| {
        DeclarchError::Other(format!("Failed to backup {}: {}", file_path.display(), e))
    })?;

    Ok(backup_path)
}

pub fn restore_from_backup(backup_path: &Path) -> Result<()> {
    let file_name = backup_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| DeclarchError::Other("Invalid backup path".to_string()))?;

    let original_name = file_name
        .split(".kdl.bak.")
        .next()
        .ok_or_else(|| DeclarchError::Other("Invalid backup filename format".to_string()))?
        .to_string()
        + ".kdl";

    let original_path = backup_path
        .parent()
        .ok_or_else(|| DeclarchError::Other("Cannot determine parent directory".to_string()))?
        .join(original_name);

    fs::copy(backup_path, &original_path).map_err(|e| {
        DeclarchError::Other(format!(
            "Failed to restore {}: {}",
            original_path.display(),
            e
        ))
    })?;

    let _ = fs::remove_file(backup_path);

    Ok(())
}
