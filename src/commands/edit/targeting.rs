use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use std::path::{Component, Path, PathBuf};

pub(super) fn resolve_target_path(config_dir: &Path, target: &str) -> Result<PathBuf> {
    validate_edit_target(target)?;
    let target_path = PathBuf::from(target);

    if target_path.components().count() == 1 {
        let mut module_path = PathBuf::from("modules").join(&target_path);

        if module_path.extension().is_none() {
            module_path.set_extension("kdl");
        }

        let full_path = config_dir.join(&module_path);

        if full_path.exists() {
            return Ok(full_path);
        }

        let modules_dir = config_dir.join("modules");
        if modules_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&modules_dir)
        {
            for category_entry in entries {
                if let Ok(category_entry) = category_entry
                    && let Ok(file_type) = category_entry.file_type()
                    && file_type.is_dir()
                {
                    let category_name = category_entry.file_name();
                    let nested_path = config_dir
                        .join("modules")
                        .join(&category_name)
                        .join(&target_path);

                    let nested_path = if nested_path.extension().is_none() {
                        let mut p = nested_path.clone();
                        p.set_extension("kdl");
                        p
                    } else {
                        nested_path
                    };

                    if nested_path.exists() {
                        output::info(&format!(
                            "Found in category: {}",
                            category_name.to_string_lossy()
                        ));
                        return Ok(nested_path);
                    }
                }
            }
        }

        return Err(DeclarchError::Other(format!(
            "Module '{}' not found\n  Tried: {}\n  Hint: Use '{}' to list available modules",
            target,
            full_path.display(),
            project_identity::cli_with("info")
        )));
    }

    let mut full_path = if target.starts_with("modules/") || target.starts_with("./") {
        config_dir.join(target)
    } else {
        config_dir.join("modules").join(target)
    };

    if full_path.extension().is_none() {
        full_path.set_extension("kdl");
    }

    Ok(full_path)
}

fn validate_edit_target(target: &str) -> Result<()> {
    if target.trim().is_empty() {
        return Err(DeclarchError::Other("Invalid module name".into()));
    }

    let path = Path::new(target);
    if path.is_absolute() {
        return Err(DeclarchError::Other(
            "Invalid module name: absolute paths are not allowed".into(),
        ));
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::RootDir) {
            return Err(DeclarchError::Other(
                "Invalid module name: path traversal is not allowed".into(),
            ));
        }
    }

    Ok(())
}
