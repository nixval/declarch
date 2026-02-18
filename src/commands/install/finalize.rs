use crate::config::editor::{self, ModuleEdit};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;

use super::file_ops::{inject_import_to_root, rollback_install_edits};

#[derive(Debug, Default)]
pub(super) struct FinalizeResult {
    pub(super) modified_modules: Vec<String>,
    pub(super) all_packages: Vec<String>,
    pub(super) root_backup: Option<std::path::PathBuf>,
}

pub(super) fn finalize_edits(all_edits: &[ModuleEdit]) -> Result<FinalizeResult> {
    let mut result = FinalizeResult::default();

    for edit in all_edits {
        if let Some(module_name) = edit.file_path.file_stem()
            && let Some(module_str) = module_name.to_str()
            && !result.modified_modules.iter().any(|m| m == module_str)
        {
            result.modified_modules.push(module_str.to_string());
        }

        if edit.created_new_file {
            let module_path = edit
                .file_path
                .strip_prefix(&paths::config_dir()?)
                .unwrap_or(&edit.file_path);
            if result.root_backup.is_none() {
                let config_path = paths::config_file()?;
                if config_path.exists() {
                    result.root_backup = Some(editor::backup_kdl_file(&config_path)?);
                }
            }
            inject_import_to_root(module_path)?;
        }

        result
            .all_packages
            .extend(edit.packages_added.iter().cloned());
    }

    Ok(result)
}

pub(super) fn run_auto_sync(
    modified_modules: &[String],
    verbose: bool,
    yes: bool,
    all_edits: &[ModuleEdit],
    root_backup: Option<&std::path::PathBuf>,
) -> Result<()> {
    use crate::commands::sync::{self, SyncOptions};

    let sync_result = sync::run(SyncOptions {
        update: false,
        prune: false,
        dry_run: false,
        verbose,
        target: None,
        yes,
        force: false,
        noconfirm: false,
        hooks: false,
        profile: None,
        host: None,
        modules: modified_modules.to_vec(),
        diff: false,
        format: None,
        output_version: None,
    });
    if verbose {
        output::verbose(&format!(
            "Auto-sync modules: {}",
            if modified_modules.is_empty() {
                "(none)".to_string()
            } else {
                modified_modules.join(", ")
            }
        ));
    }

    match sync_result {
        Ok(()) => {
            if output::is_interrupted() {
                rollback_install_edits(all_edits, root_backup);
                output::info("Changes rolled back");
                return Ok(());
            }
        }
        Err(DeclarchError::Interrupted) => {
            rollback_install_edits(all_edits, root_backup);
            output::info("Changes rolled back");
            return Ok(());
        }
        Err(e) => {
            rollback_install_edits(all_edits, root_backup);
            return Err(e);
        }
    }

    Ok(())
}
