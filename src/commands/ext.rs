use crate::error::Result;
use crate::ui as output;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

/// Hidden placeholder for future extension protocol.
pub fn run() -> Result<()> {
    output::header("Extension Protocol Placeholder");
    output::info("External extension runtime (execution) is not implemented yet.");
    output::info("Planned discovery pattern: declarch-ext-*");
    output::info("Planned contract version: v1");
    output::info("See: docs/contracts/v1/README.md");

    let discovered = discover_extensions_from_path();
    if discovered.is_empty() {
        output::info("No extensions discovered in PATH.");
    } else {
        output::info("Discovered extensions:");
        for ext in discovered {
            output::indent(&format!("• {}", ext), 2);
        }
    }
    Ok(())
}

fn discover_extensions_from_path() -> Vec<String> {
    let mut found = BTreeSet::new();
    let Some(path_var) = env::var_os("PATH") else {
        return Vec::new();
    };

    for dir in env::split_paths(&path_var) {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_executable_candidate(&path) {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let normalized = normalize_ext_name(name);
            if normalized.starts_with("declarch-ext-") {
                found.insert(normalized);
            }
        }
    }

    found.into_iter().collect()
}

fn is_executable_candidate(path: &Path) -> bool {
    path.is_file()
}

fn normalize_ext_name(name: &str) -> String {
    if cfg!(windows) {
        for suffix in [".exe", ".cmd", ".bat", ".ps1"] {
            if let Some(base) = name.strip_suffix(suffix) {
                return base.to_string();
            }
        }
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_ext_name;

    #[test]
    fn normalize_keeps_unix_name() {
        assert_eq!(
            normalize_ext_name("declarch-ext-security"),
            "declarch-ext-security"
        );
    }

    #[test]
    fn normalize_windows_suffixes() {
        if cfg!(windows) {
            assert_eq!(
                normalize_ext_name("declarch-ext-security.exe"),
                "declarch-ext-security"
            );
            assert_eq!(
                normalize_ext_name("declarch-ext-security.cmd"),
                "declarch-ext-security"
            );
        }
    }
}
