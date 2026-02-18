use crate::config::editor::{ModuleEdit, restore_from_backup};
use crate::error::{DeclarchError, Result};
use crate::utils::paths;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn rollback_install_edits(all_edits: &[ModuleEdit], root_backup: Option<&PathBuf>) {
    for edit in all_edits {
        if let Some(ref backup) = edit.backup_path {
            let _ = restore_from_backup(backup);
        } else if edit.created_new_file {
            let _ = fs::remove_file(&edit.file_path);
        }
    }

    if let Some(backup) = root_backup {
        let _ = restore_from_backup(backup);
    }
}

pub(super) fn cleanup_install_backups(all_edits: &[ModuleEdit], root_backup: Option<&PathBuf>) {
    for edit in all_edits {
        if let Some(ref backup) = edit.backup_path {
            let _ = fs::remove_file(backup);
        }
    }

    if let Some(backup) = root_backup {
        let _ = fs::remove_file(backup);
    }
}

/// Auto-import newly created modules into root config.
pub(super) fn inject_import_to_root(module_path: &Path) -> Result<()> {
    let config_path = paths::config_file()?;

    let import_path = module_path
        .components()
        .map(|comp| comp.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;

    if content.contains(&import_path) {
        return Ok(());
    }

    let import_line = format!("    {:?}", import_path);

    let re = Regex::new(r#"(?m)^(.*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        re.replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        })
        .to_string()
    } else {
        format!(
            "{}\n\nimports {{\n{}\n}}\n",
            content.trim_end(),
            import_line
        )
    };

    fs::write(&config_path, new_content)?;

    Ok(())
}
