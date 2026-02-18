use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::templates;
use colored::Colorize;
use kdl::KdlDocument;
use std::path::Path;

pub(super) fn create_module_from_template(file_path: &Path, validate_only: bool) -> Result<()> {
    let module_name = file_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if module_name.is_empty() {
        return Err(DeclarchError::Other("Invalid module name".into()));
    }

    if file_path.exists() {
        output::info(&format!(
            "Module '{}' already exists, opening for editing...",
            file_path.display()
        ));
        return Ok(());
    }

    output::header("Creating New Module");
    output::info(&format!("Module: {}", module_name.cyan()));
    output::info(&format!("Path: {}", file_path.display().to_string().cyan()));

    let template = templates::get_template_by_name(&module_name)
        .unwrap_or_else(|| templates::default_module(&module_name));

    if let Err(e) = template.parse::<KdlDocument>() {
        return Err(DeclarchError::Other(format!(
            "Template KDL is invalid: {}\nPlease report this bug.",
            e
        )));
    }

    if validate_only {
        output::success("Template is valid KDL. File would be created.");
        return Ok(());
    }

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(file_path, template)?;
    output::success(&format!("Created module: {}", file_path.display()));

    Ok(())
}

pub(super) fn validate_file_only(file_path: &Path) -> Result<()> {
    output::header("Validating Configuration");
    output::info(&format!("File: {}", file_path.display().to_string().cyan()));

    let content = std::fs::read_to_string(file_path)?;

    match content.parse::<KdlDocument>() {
        Ok(_) => {
            output::success("KDL syntax is valid!");
            Ok(())
        }
        Err(e) => {
            output::error("KDL syntax error detected!");
            output::error(&format!("  {}", e));
            std::process::exit(1);
        }
    }
}

pub(super) fn create_backup(file_path: &Path) -> Result<()> {
    use chrono::Local;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = file_path.with_extension(format!("kdl.backup.{}", timestamp));

    std::fs::copy(file_path, &backup_path)?;
    output::info(&format!("Backup created: {}", backup_path.display()));

    Ok(())
}

pub(super) fn format_kdl_file(file_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file_path)?;

    match content.parse::<KdlDocument>() {
        Ok(doc) => {
            let formatted = doc.to_string();

            if formatted != content {
                std::fs::write(file_path, formatted)?;
                output::info("Auto-formatted KDL");
            }
            Ok(())
        }
        Err(e) => {
            output::warning(&format!("Cannot format invalid KDL: {}", e));
            Ok(())
        }
    }
}
