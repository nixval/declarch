mod file_ops;
mod targeting;

use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use file_ops::{create_backup, create_module_from_template, format_kdl_file, validate_file_only};
use kdl::KdlDocument;
use std::process::Command;
use targeting::resolve_target_path;
use which;

#[derive(Debug)]
pub struct EditOptions {
    pub target: Option<String>,
    pub dry_run: bool,
    pub preview: bool,
    pub number: bool,
    pub create: bool,
    pub auto_format: bool,
    pub validate_only: bool,
    pub backup: bool,
}

pub fn run(options: EditOptions) -> Result<()> {
    let config_dir = paths::config_dir()?;

    // Check if declarch is initialized
    if !config_dir.exists() {
        return Err(DeclarchError::Other(format!(
            "{} not initialized. Run '{}' first.",
            project_identity::DISPLAY_NAME,
            project_identity::cli_with("init")
        )));
    }

    // Determine which file to edit
    let file_to_edit = if let Some(target) = options.target {
        // User specified a target (module path or filename)
        resolve_target_path(&config_dir, &target)?
    } else {
        // No target specified, edit root config
        paths::config_file()?
    };

    // Handle --create: Create new module from template if it doesn't exist
    if options.create && !file_to_edit.exists() {
        return create_module_from_template(&file_to_edit, options.validate_only);
    }

    // Handle --validate-only: Only check syntax and exit
    if options.validate_only {
        return validate_file_only(&file_to_edit);
    }

    // Verify file exists (after handling --create)
    if !file_to_edit.exists() {
        return Err(DeclarchError::Other(format!(
            "File not found: {}\nHint: Use '{}' first, or use --create to make a new module",
            file_to_edit.display(),
            project_identity::cli_with("init")
        )));
    }

    // Handle --format: Auto-format KDL before opening
    if options.auto_format {
        format_kdl_file(&file_to_edit)?;
    }

    // Handle preview mode (like cat)
    if options.preview {
        output::header("Preview Configuration");
        output::info(&format!(
            "File: {}",
            file_to_edit.display().to_string().cyan()
        ));
        println!();

        let content = std::fs::read_to_string(&file_to_edit)?;

        if options.number {
            // Show with line numbers
            for (line_num, line) in content.lines().enumerate() {
                println!("{:4} │ {}", line_num + 1, line);
            }
        } else {
            // Plain output
            print!("{}", content);
            // Ensure trailing newline
            if !content.ends_with('\n') {
                println!();
            }
        }

        println!();
        output::success(&format!("{} lines", content.lines().count()));
        return Ok(());
    }

    // Get editor with fallback chain
    let (editor, editor_source, was_fallback) = get_editor_with_fallback()?;

    // Handle dry-run mode
    if options.dry_run {
        output::header("Dry Run: Edit Configuration");
        output::info(&format!(
            "Would open: {}",
            file_to_edit.display().to_string().cyan()
        ));
        output::info(&format!(
            "With editor: {} (from {})",
            editor.green(),
            editor_source
        ));
        return Ok(());
    }

    // Handle --backup: Create backup before editing
    if options.backup && file_to_edit.exists() {
        create_backup(&file_to_edit)?;
    }

    // Show info with editor source
    output::header("Editing Configuration");
    output::info(&format!(
        "File: {}",
        file_to_edit.display().to_string().cyan()
    ));

    if was_fallback {
        output::info(&format!("Editor: {} (fallback)", editor.green()));
    } else {
        output::info(&format!(
            "Editor: {} (from {})",
            editor.green(),
            editor_source
        ));
    }

    // Open editor
    let status = Command::new(&editor)
        .arg(&file_to_edit)
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: editor.clone(),
            reason: e.to_string(),
        })?;

    if !status.success() {
        return Err(DeclarchError::Other(format!(
            "Editor exited with non-zero status: {:?}",
            status.code()
        )));
    }

    // Verify syntax after editing
    output::info("Verifying configuration syntax...");
    let content = std::fs::read_to_string(&file_to_edit)?;

    // Simple KDL syntax check
    if let Err(e) = content.parse::<KdlDocument>() {
        output::warning("KDL syntax error detected!");
        output::warning(&format!("  {}", e));
        output::info(&format!(
            "Run '{}' for more details",
            project_identity::cli_with("lint --mode validate")
        ));
    } else {
        output::success("Configuration syntax is valid!");
    }

    Ok(())
}

/// Check if editor binary exists in PATH
fn editor_exists(editor: &str) -> bool {
    // Handle editors with arguments (e.g., "code -w")
    let binary = editor.split_whitespace().next().unwrap_or(editor);
    which::which(binary).is_ok()
}

/// Get editor with validation and fallback chain
///
/// Tries editors in priority order until finding one that exists:
/// 1. KDL config 'editor' field (declarative)
/// 2. $VISUAL environment variable (GUI editors)
/// 3. $EDITOR environment variable (terminal editors)
/// 4. "nano" (fallback)
///
/// Returns: (editor, source, was_fallback)
fn get_editor_with_fallback() -> Result<(String, &'static str, bool)> {
    // Priority 1: KDL config 'editor' field
    if let Ok(config) =
        crate::config::loader::load_root_config(&crate::utils::paths::config_file()?)
        && let Some(ref editor) = config.editor
    {
        if editor_exists(editor) {
            return Ok((editor.clone(), "config", false));
        } else {
            output::warning(&format!("Configured editor '{}' not found in PATH", editor));
        }
    }

    // Priority 2: $VISUAL environment variable
    if let Ok(ed) = std::env::var("VISUAL")
        && !ed.is_empty()
        && editor_exists(&ed)
    {
        return Ok((ed, "$VISUAL", false));
    }

    // Priority 3: $EDITOR environment variable
    if let Ok(ed) = std::env::var("EDITOR")
        && !ed.is_empty()
        && editor_exists(&ed)
    {
        return Ok((ed, "$EDITOR", false));
    }

    // Priority 4: Fallback to nano
    if editor_exists("nano") {
        output::info("Falling back to 'nano' editor");
        Ok(("nano".to_string(), "default", true))
    } else {
        Err(DeclarchError::Other(
            "No editor found. Please install nano or set $EDITOR environment variable.".into(),
        ))
    }
}

#[cfg(test)]
mod tests;
