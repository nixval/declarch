//! Settings Module
//!
//! Manages declarch configuration settings (color, progress, format, etc.)

use crate::error::{DeclarchError, Result};
use crate::utils::paths;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Settings manager
pub struct Settings {
    settings_file: PathBuf,
    values: HashMap<String, String>,
}

impl Settings {
    /// Load settings from file, or create default
    pub fn load() -> Result<Self> {
        let settings_file = paths::config_dir()?.join("settings.kdl");

        let values = if settings_file.exists() {
            self::parse_settings(&settings_file)?
        } else {
            Self::defaults()
        };

        Ok(Self {
            settings_file,
            values,
        })
    }

    /// Get a setting value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    /// Set a setting value
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // Validate setting key
        self.validate_key(&key)?;

        // Validate setting value
        self.validate_value(&key, &value)?;

        // Update value
        self.values.insert(key, value);

        // Save to file
        self.save()?;

        Ok(())
    }

    /// Reset a setting to default
    pub fn reset(&mut self, key: &str) -> Result<()> {
        if let Some(default) = Self::defaults().get(key) {
            self.values.insert(key.to_string(), default.clone());
            self.save()?;
        } else {
            return Err(DeclarchError::Other(format!("Unknown setting: '{}'", key)));
        }

        Ok(())
    }

    /// Get all settings
    pub fn all(&self) -> &HashMap<String, String> {
        &self.values
    }

    /// Validate setting key
    fn validate_key(&self, key: &str) -> Result<()> {
        let valid_keys = [
            "color", "progress", "format", "verbose", "editor", "compact",
        ];

        if !valid_keys.contains(&key) {
            return Err(DeclarchError::Other(format!(
                "Unknown setting: '{}'. Valid settings: {}",
                key,
                valid_keys.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate setting value
    fn validate_value(&self, key: &str, value: &str) -> Result<()> {
        match key {
            "color" => {
                let valid = ["auto", "always", "never"];
                if !valid.contains(&value) {
                    return Err(DeclarchError::Other(format!(
                        "Invalid value for '{}': '{}'. Valid: {}",
                        key,
                        value,
                        valid.join(", ")
                    )));
                }
            }
            "progress" => {
                let valid = ["on", "off"];
                if !valid.contains(&value) {
                    return Err(DeclarchError::Other(format!(
                        "Invalid value for '{}': '{}'. Valid: {}",
                        key,
                        value,
                        valid.join(", ")
                    )));
                }
            }
            "format" => {
                let valid = ["table", "json", "yaml"];
                if !valid.contains(&value) {
                    return Err(DeclarchError::Other(format!(
                        "Invalid value for '{}': '{}'. Valid: {}",
                        key,
                        value,
                        valid.join(", ")
                    )));
                }
            }
            "verbose" => {
                let valid = ["true", "false"];
                if !valid.contains(&value) {
                    return Err(DeclarchError::Other(format!(
                        "Invalid value for '{}': '{}'. Valid: {}",
                        key,
                        value,
                        valid.join(", ")
                    )));
                }
            }
            "editor" => {
                // Allow any non-empty string (editor command)
                if value.trim().is_empty() {
                    return Err(DeclarchError::Other("Editor cannot be empty".to_string()));
                }
            }
            "compact" => {
                let valid = ["true", "false"];
                if !valid.contains(&value) {
                    return Err(DeclarchError::Other(format!(
                        "Invalid value for '{}': '{}'. Valid: {}",
                        key,
                        value,
                        valid.join(", ")
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Save settings to file
    fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DeclarchError::Other(format!("Failed to create settings directory: {}", e))
            })?;
        }

        // Write settings in KDL format
        let mut content = String::from(
            "// Declarch settings\n// Generated by declarch settings command\n\nsettings {\n",
        );

        for (key, value) in &self.values {
            content.push_str(&format!("  {} {}\n", key, value));
        }

        content.push_str("}\n");

        fs::write(&self.settings_file, content)
            .map_err(|e| DeclarchError::Other(format!("Failed to save settings: {}", e)))?;

        Ok(())
    }

    /// Get default settings
    fn defaults() -> HashMap<String, String> {
        let mut defaults = HashMap::new();
        defaults.insert("color".to_string(), "auto".to_string());
        defaults.insert("progress".to_string(), "off".to_string());
        defaults.insert("format".to_string(), "table".to_string());
        defaults.insert("verbose".to_string(), "false".to_string());
        defaults.insert("editor".to_string(), "".to_string()); // Empty = use system default
        defaults.insert("compact".to_string(), "false".to_string()); // Compact mode disabled by default
        defaults
    }
}

/// Parse settings from KDL file
fn parse_settings(path: &PathBuf) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read settings: {}", e)))?;

    let mut values = HashMap::new();

    // Simple parser: look for "settings { key value }" pattern
    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        // Look for key value pairs inside settings block
        if !line.starts_with("settings") && line != "}" && line != "{" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                values.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    // Merge with defaults to ensure all keys exist
    let defaults = Settings::defaults();
    for (key, default_value) in defaults {
        if !values.contains_key(&key) {
            values.insert(key, default_value);
        }
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let defaults = Settings::defaults();
        assert_eq!(defaults.get("color"), Some(&"auto".to_string()));
        assert_eq!(defaults.get("progress"), Some(&"off".to_string()));
        assert_eq!(defaults.get("format"), Some(&"table".to_string()));
        assert_eq!(defaults.get("verbose"), Some(&"false".to_string()));
        assert_eq!(defaults.get("compact"), Some(&"false".to_string()));
    }

    #[test]
    fn test_validate_key_valid() {
        let settings = Settings {
            settings_file: PathBuf::from("/tmp/test.kdl"),
            values: HashMap::new(),
        };

        assert!(settings.validate_key("color").is_ok());
        assert!(settings.validate_key("progress").is_ok());
        assert!(settings.validate_key("format").is_ok());
        assert!(settings.validate_key("verbose").is_ok());
    }

    #[test]
    fn test_validate_key_invalid() {
        let settings = Settings {
            settings_file: PathBuf::from("/tmp/test.kdl"),
            values: HashMap::new(),
        };

        assert!(settings.validate_key("invalid").is_err());
    }

    #[test]
    fn test_validate_value_color() {
        let settings = Settings {
            settings_file: PathBuf::from("/tmp/test.kdl"),
            values: HashMap::new(),
        };

        assert!(settings.validate_value("color", "auto").is_ok());
        assert!(settings.validate_value("color", "always").is_ok());
        assert!(settings.validate_value("color", "never").is_ok());
        assert!(settings.validate_value("color", "invalid").is_err());
    }
}
