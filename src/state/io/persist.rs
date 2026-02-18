use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::state::types::State;
use crate::ui;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::CURRENT_STATE_SCHEMA_VERSION;
use super::backup_ops::rotate_backups;

pub(super) fn prepare_and_write_state(
    state: &State,
    path: &Path,
    sync_dir_after_write: bool,
) -> Result<()> {
    let state = normalize_state_for_persist(state);
    write_state_atomically(path, &state, sync_dir_after_write)
}

fn normalize_state_for_persist(state: &State) -> State {
    let mut state = state.clone();
    state.meta.schema_version = CURRENT_STATE_SCHEMA_VERSION;
    state.meta.state_revision = Some(state.meta.state_revision.unwrap_or(0) + 1);
    if state.meta.generator.is_none() {
        state.meta.generator = Some(project_identity::STABLE_PROJECT_ID.to_string());
    }
    state
}

fn write_state_atomically(path: &Path, state: &State, sync_dir_after_write: bool) -> Result<()> {
    let dir = path.parent().ok_or_else(|| {
        DeclarchError::PathError(format!(
            "Invalid state path (no parent directory): {}",
            path.display()
        ))
    })?;

    rotate_backups(dir, path)?;

    let content = serde_json::to_string_pretty(state)
        .map_err(|e| DeclarchError::SerializationError(format!("State serialization: {}", e)))?;

    let _: State = serde_json::from_str(&content)
        .map_err(|e| DeclarchError::SerializationError(format!("Invalid JSON generated: {}", e)))?;

    let tmp_path = dir.join("state.tmp");
    let mut tmp_file = fs::File::create(&tmp_path).map_err(|e| DeclarchError::IoError {
        path: tmp_path.clone(),
        source: e,
    })?;

    tmp_file.write_all(content.as_bytes())?;
    tmp_file.sync_all()?;
    drop(tmp_file);

    fs::rename(&tmp_path, path).map_err(|e| DeclarchError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;

    if sync_dir_after_write
        && let Ok(dir_file) = fs::File::open(dir)
        && let Err(e) = dir_file.sync_all()
    {
        ui::warning(&format!("Failed to sync state directory: {}", e));
    }

    Ok(())
}
