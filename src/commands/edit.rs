use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use kdl::KdlDocument;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct EditOptions {
    pub target: Option<String>,
    pub dry_run: bool,
}

pub fn run(options: EditOptions) -> Result<()> {
    let config_dir = paths::config_dir()?;

    // Check if declarch is initialized
    if !config_dir.exists() {
        return Err(DeclarchError::Other(
            "Declarch not initialized. Run 'declarch init' first.".into(),
        ));
    }

    // Determine which file to edit
    let file_to_edit = if let Some(target) = options.target {
        // User specified a target (module path or filename)
        resolve_target_path(&config_dir, &target)?
    } else {
        // No target specified, edit root config
        paths::config_file()?
    };

    // Verify file exists
    if !file_to_edit.exists() {
        return Err(DeclarchError::Other(format!(
            "File not found: {}\nHint: Use 'declarch init' first or check the module path",
            file_to_edit.display()
        )));
    }

    // Handle dry-run mode
    if options.dry_run {
        output::header("Dry Run: Edit Configuration");
        let editor = get_editor_from_config()?;
        output::info(&format!(
            "Would open: {}",
            file_to_edit.display().to_string().cyan()
        ));
        output::info(&format!("With editor: {}", editor.green()));
        return Ok(());
    }

    // Get editor from config file or environment
    let editor = get_editor_from_config()?;

    // Show info
    output::header("Editing Configuration");
    output::info(&format!(
        "File: {}",
        file_to_edit.display().to_string().cyan()
    ));
    output::info(&format!("Editor: {}", editor.green()));

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
        output::info("Run 'declarch check' for more details");
    } else {
        output::success("Configuration syntax is valid!");
    }

    Ok(())
}

/// Resolve target path to actual file location
///
/// Examples:
/// - "hyprland/niri-nico" → modules/hyprland/niri-nico.kdl
/// - "declarch" → declarch.kdl
/// - "modules/hyprland/niri-nico.kdl" → modules/hyprland/niri-nico.kdl
fn resolve_target_path(config_dir: &Path, target: &str) -> Result<PathBuf> {
    let target_path = PathBuf::from(target);

    // If target is just a filename (no slashes), check in modules/
    if target_path.components().count() == 1 {
        let mut module_path = PathBuf::from("modules").join(&target_path);

        // Add .kdl extension if not present
        if module_path.extension().is_none() {
            module_path.set_extension("kdl");
        }

        let full_path = config_dir.join(&module_path);

        // If exact path exists, use it
        if full_path.exists() {
            return Ok(full_path);
        }

        // Try with category prefix (e.g., "niri-nico" → "hyprland/niri-nico")
        // Search through all categories
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

                    // Add .kdl extension if needed
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

        // Not found
        return Err(DeclarchError::Other(format!(
            "Module '{}' not found\n  Tried: {}\n  Hint: Use 'declarch info' to list available modules",
            target,
            full_path.display() // Use full_path which already includes modules/
        )));
    }

    // Target has slashes (e.g., "hyprland/niri-nico" or "modules/hyprland/niri-nico")
    let mut full_path = if target.starts_with("modules/") || target.starts_with("./") {
        config_dir.join(target)
    } else {
        config_dir.join("modules").join(target)
    };

    // Add .kdl extension if not present
    if full_path.extension().is_none() {
        full_path.set_extension("kdl");
    }

    Ok(full_path)
}

/// Get editor to use from settings or environment
///
/// Priority:
/// 1. Settings (declarch settings set editor nvim)
/// 2. $EDITOR environment variable
/// 3. $VISUAL environment variable
/// 4. "nano" (default fallback)
fn get_editor_from_config() -> Result<String> {
    // Priority 1: Settings system (NEW)
    if let Ok(settings) = crate::config::settings::Settings::load() {
        if let Some(editor) = settings.get("editor") {
            if !editor.is_empty() {
                return Ok(editor.clone());
            }
        }
    }

    // Priority 2: Check environment variables
    if let Ok(ed) = std::env::var("EDITOR")
        && !ed.is_empty()
    {
        return Ok(ed);
    }
    if let Ok(ed) = std::env::var("VISUAL")
        && !ed.is_empty()
    {
        return Ok(ed);
    }

    // Priority 3: Fallback to nano
    Ok("nano".to_string())
}
