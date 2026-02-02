//! Error suggestions and helpful messages
//!
//! Provides actionable suggestions for common error scenarios

use crate::error::DeclarchError;
use colored::Colorize;

/// Extension trait to add suggestions to errors
pub trait ErrorSuggestions {
    /// Get a helpful suggestion for this error
    fn suggestion(&self) -> Option<String>;

    /// Display the error with its suggestion
    fn display_with_suggestion(&self) -> String;
}

impl ErrorSuggestions for DeclarchError {
    fn suggestion(&self) -> Option<String> {
        match self {
            DeclarchError::ConfigNotFound { path: _ } => Some(format!(
                "Run '{}' to create initial configuration\nOr specify a different config path",
                "declarch init".cyan()
            )),
            DeclarchError::ConfigError(msg) => {
                if msg.contains("KDL") {
                    Some(format!(
                        "Check your KDL syntax at {}\nCommon issues: missing braces, invalid node names",
                        "https://kdl.dev".cyan()
                    ))
                } else {
                    Some(format!(
                        "Run '{}' to verify your configuration",
                        "declarch check validate".cyan()
                    ))
                }
            }
            DeclarchError::TargetNotFound(target) => Some(format!(
                "Package '{}' not found in any backend\nTry: {} or {}",
                target,
                format!("declarch search {}", target).cyan(),
                format!("declarch info --package {}", target).cyan()
            )),
            DeclarchError::DependencyMissing(dep) => Some(format!(
                "Install '{}' first, then retry the operation",
                dep.cyan()
            )),
            DeclarchError::SystemCommandFailed { command, reason: _ } => {
                if command.contains("pacman") || command.contains("paru") || command.contains("yay")
                {
                    Some(format!(
                        "Check your AUR helper configuration: {}\nCommon fixes: {}, {}",
                        "declarch settings show".cyan(),
                        "sudo pacman -Sy".cyan(),
                        "paru -Syu".cyan()
                    ))
                } else if command.contains("flatpak") {
                    Some(format!(
                        "Ensure flatpak is installed: {}\nOr check remote configuration",
                        "sudo pacman -S flatpak".cyan()
                    ))
                } else {
                    Some(format!(
                        "Check that '{}' is installed and in your PATH",
                        command.split_whitespace().next().unwrap_or(command).cyan()
                    ))
                }
            }
            DeclarchError::PackageManagerError(msg) => {
                if msg.contains("not found") {
                    Some(format!(
                        "The package might not exist or might have a different name\nTry: {}",
                        "declarch search <package-name>".cyan()
                    ))
                } else if msg.contains("already installed") {
                    Some(format!(
                        "Package is already installed. Use '{}' to update or '{}' to reinstall",
                        "declarch sync --update".cyan(),
                        "declarch sync --force".cyan()
                    ))
                } else if msg.contains("lock") || msg.contains("database") {
                    Some(format!(
                        "Another package manager process is running\nTry: {}",
                        "sudo rm /var/lib/pacman/db.lck".red()
                    ))
                } else {
                    None
                }
            }
            DeclarchError::ParseError { file, message } => Some(format!(
                "Check {} for syntax errors\nError: {}",
                file.cyan(),
                message
            )),
            DeclarchError::Interrupted => Some(format!(
                "Operation was cancelled. Run '{}' to resume",
                "declarch sync".cyan()
            )),
            _ => None,
        }
    }

    fn display_with_suggestion(&self) -> String {
        let base_msg = format!("{}", self);

        if let Some(suggestion) = self.suggestion() {
            format!("{}\n\n{}", base_msg.red().bold(), suggestion)
        } else {
            base_msg.red().bold().to_string()
        }
    }
}

/// Display an error with its suggestion to the user
pub fn display_error_with_suggestion(error: &DeclarchError) {
    eprintln!("{}", error.display_with_suggestion());
}

/// Get suggestions for common CLI mistakes
pub fn get_cli_suggestion(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();

    if input_lower.contains("install") && input_lower.contains("package") {
        Some(format!(
            "Did you mean: {} or {}?",
            "declarch install <package>".cyan(),
            "declarch sync".cyan()
        ))
    } else if input_lower.contains("remove") || input_lower.contains("delete") {
        Some(format!(
            "To remove packages:\n1. Remove from config files\n2. Run {}",
            "declarch sync --prune".cyan()
        ))
    } else if input_lower.contains("update") || input_lower.contains("upgrade") {
        Some(format!(
            "Did you mean: {}?",
            "declarch sync --update".cyan()
        ))
    } else if input_lower.contains("list") || input_lower.contains("show") {
        Some(format!(
            "Try: {}, {}, or {}",
            "declarch list".cyan(),
            "declarch info".cyan(),
            "declarch info --summary".cyan()
        ))
    } else {
        None
    }
}

/// Provide package name suggestions when a package is not found
pub fn suggest_package_name(package_name: &str, available_packages: &[String]) -> Option<String> {
    use std::collections::HashMap;

    // First check if there's an exact match - if so, don't suggest anything
    if available_packages.iter().any(|p| p == package_name) {
        return None;
    }

    // Calculate Levenshtein distance for each available package
    let mut distances: HashMap<&String, usize> = HashMap::new();

    for available in available_packages {
        let dist = levenshtein_distance(package_name, available);
        distances.insert(available, dist);
    }

    // Find the closest match with lowest distance
    let mut closest: Option<(&String, usize)> = None;
    for (pkg, dist) in distances {
        if dist < closest.map(|(_, d)| d).unwrap_or(usize::MAX) {
            closest = Some((pkg, dist));
        }
    }

    // Only suggest if the distance is reasonable (<= 3) and not exact match (dist > 0)
    if let Some((closest_pkg, dist)) = closest {
        if dist > 0 && dist <= 3 {
            return Some(format!("Did you mean: {}?", closest_pkg.cyan()));
        }
    }

    None
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };

            curr_row[j + 1] = (prev_row[j + 1] + 1)
                .min(curr_row[j] + 1)
                .min(prev_row[j] + cost);
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("bat", "cat"), 1);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_suggest_package_name() {
        let available = vec![
            "bat".to_string(),
            "cat".to_string(),
            "fzf".to_string(),
            "ripgrep".to_string(),
        ];

        // Should suggest "bat" for "bta" (typo)
        let suggestion = suggest_package_name("bta", &available);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("bat"));

        // Should not suggest for exact match
        let suggestion = suggest_package_name("bat", &available);
        assert!(suggestion.is_none());

        // Should not suggest for very different names
        let suggestion = suggest_package_name("firefox", &available);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_config_not_found_suggestion() {
        let error = DeclarchError::ConfigNotFound {
            path: std::path::PathBuf::from("/fake/path"),
        };

        let suggestion = error.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("declarch init"));
    }

    #[test]
    fn test_target_not_found_suggestion() {
        let error = DeclarchError::TargetNotFound("firefox".to_string());

        let suggestion = error.suggestion();
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert!(s.contains("declarch search"));
        assert!(s.contains("firefox"));
    }
}
