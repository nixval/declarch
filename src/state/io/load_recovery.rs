use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::state::types::State;
use crate::ui;
use std::fs;
use std::path::{Path, PathBuf};

use super::backup_ops::restore_from_backup;
use super::migration::migrate_state;
use super::{save_state, validate_state_integrity};

pub(super) fn load_state_from_path(path: &Path, strict_recovery: bool) -> Result<State> {
    let path = path.to_path_buf();
    if !path.exists() {
        return Ok(State::default());
    }

    let content = fs::read_to_string(&path);
    let state = match content {
        Ok(content) => match serde_json::from_str::<State>(&content) {
            Ok(mut state) => {
                let issues = validate_state_integrity(&state);
                if !issues.is_empty() {
                    ui::warning("State integrity issues detected:");
                    for issue in &issues {
                        ui::indent(&format!("• {}", issue), 2);
                    }
                }

                if migrate_state(&mut state)? {
                    ui::info("State migrated to fix duplicate keys");
                    let _ = save_state(&state);
                }
                return Ok(state);
            }
            Err(e) => {
                ui::error(&format!("State file corrupted: {}", e));
                ui::info("Attempting to restore from backup...");
                recover_or_default(
                    &path,
                    strict_recovery,
                    "State file is corrupted and backup restore failed (strict mode).",
                    "inspect state backups",
                )?
            }
        },
        Err(e) => {
            ui::error(&format!("Failed to read state file: {}", e));
            ui::info("Attempting to restore from backup...");
            recover_or_default(
                &path,
                strict_recovery,
                "State file cannot be read and backup restore failed (strict mode).",
                "inspect file permissions/state path",
            )?
        }
    };

    Ok(state)
}

fn recover_or_default(
    path: &PathBuf,
    strict_recovery: bool,
    strict_message: &str,
    strict_hint_fragment: &str,
) -> Result<State> {
    match restore_from_backup(path)? {
        Some(state) => {
            ui::success("State restored from backup successfully");
            Ok(state)
        }
        None => {
            ui::warning("Failed to restore from backup: no valid backup found");
            if strict_recovery {
                return Err(DeclarchError::Other(format!(
                    "{strict_message}\n\
                     File: {}\n\
                     Hint: run `{}`, {}, then retry.",
                    path.display(),
                    project_identity::cli_with("info --doctor"),
                    strict_hint_fragment,
                )));
            }
            ui::info("Using default state");
            Ok(State::default())
        }
    }
}
