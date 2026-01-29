//! Settings Command
//!
//! Manage declarch configuration settings.

use crate::config::settings::Settings;
use crate::error::Result;
use crate::ui as output;

/// Settings command
#[derive(Debug)]
pub enum SettingsCommand {
    /// Set a setting value
    Set { key: String, value: String },
    /// Get a setting value
    Get { key: String },
    /// Show all settings
    Show,
    /// Reset setting to default
    Reset { key: String },
}

/// Run the settings command
pub fn run(command: SettingsCommand) -> Result<()> {
    match command {
        SettingsCommand::Set { key, value } => {
            let mut settings = Settings::load()?;
            settings.set(key.clone(), value.clone())?;
            output::success(&format!("Set {} = {}", key, value));
        }
        SettingsCommand::Get { key } => {
            let settings = Settings::load()?;
            match settings.get(&key) {
                Some(value) => {
                    println!("{} = {}", key, value);
                }
                None => {
                    output::error(&format!("Setting '{}' not found", key));
                    return Err(crate::error::DeclarchError::Other(format!(
                        "Setting '{}' not found",
                        key
                    )));
                }
            }
        }
        SettingsCommand::Show => {
            let settings = Settings::load()?;
            show_all_settings(&settings);
        }
        SettingsCommand::Reset { key } => {
            let mut settings = Settings::load()?;
            settings.reset(&key)?;
            output::success(&format!("Reset {} to default", key));
        }
    }

    Ok(())
}

/// Show all settings
fn show_all_settings(settings: &Settings) {
    output::header("Current Settings");

    let all = settings.all();

    // Display in a consistent order
    let keys = vec!["color", "progress", "format", "verbose", "editor"];

    for key in keys {
        if let Some(value) = all.get(key) {
            // Don't show empty editor value
            if key == "editor" && value.is_empty() {
                println!("  {}: (system default)", key);
            } else {
                println!("  {}: {}", key, value);
            }
        }
    }

    println!();
    output::info("Valid values:");
    println!("  color:    auto, always, never");
    println!("  progress: on, off");
    println!("  format:   table, json, yaml");
    println!("  verbose:  true, false");
    println!("  editor:   <editor-command> (e.g., nvim, code, vim)");
}
