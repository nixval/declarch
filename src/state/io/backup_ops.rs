use crate::error::{DeclarchError, Result};
use crate::state::types::State;
use crate::ui;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn restore_from_backup(state_path: &PathBuf) -> Result<Option<State>> {
    let dir = state_path.parent().ok_or_else(|| {
        DeclarchError::PathError(format!(
            "Invalid state path (no parent directory): {}",
            state_path.display()
        ))
    })?;

    for i in 1..=3 {
        let backup_path = dir.join(format!("state.json.bak.{}", i));
        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path).map_err(|e| DeclarchError::IoError {
                path: backup_path.clone(),
                source: e,
            })?;

            match serde_json::from_str::<State>(&content) {
                Ok(state) => {
                    let _ = fs::copy(&backup_path, state_path);
                    return Ok(Some(state));
                }
                Err(_) => continue,
            }
        }
    }

    Ok(None)
}

pub(super) fn rotate_backups(dir: &Path, path: &Path) -> Result<()> {
    if path.exists() {
        let max_backups = 3;
        for i in (1..max_backups).rev() {
            let old_bak = dir.join(format!("state.json.bak.{}", i));
            let new_bak = dir.join(format!("state.json.bak.{}", i + 1));
            if old_bak.exists()
                && let Err(e) = fs::rename(&old_bak, &new_bak)
            {
                ui::warning(&format!(
                    "Failed to rotate backup {} -> {}: {}",
                    old_bak.display(),
                    new_bak.display(),
                    e
                ));
            }
        }

        let first_bak = dir.join("state.json.bak.1");
        if let Err(e) = fs::copy(path, &first_bak) {
            return Err(DeclarchError::IoError {
                path: first_bak,
                source: e,
            });
        }
    }

    Ok(())
}
