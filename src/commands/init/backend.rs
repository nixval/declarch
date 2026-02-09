//! Backend initialization logic
//!
//! Handles the `declarch init --backend <name>` command flow:
//! 1. Fetch backend definition from remote registry
//! 2. Validate and sanitize backend name
//! 3. Write backend file to `backends/<name>.kdl`
//! 4. Add import to `backends.kdl`

use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use crate::utils::{self, paths, remote};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Official backends that are embedded in the default backends.kdl
pub const OFFICIAL_BACKENDS: &[&str] = &["aur", "pacman", "flatpak"];

/// Result of attempting to add backend import
#[derive(Debug, PartialEq)]
pub enum ImportResult {
    /// Import was successfully added
    Added,
    /// Backend already defined inline (aur, pacman, flatpak)
    AlreadyExistsInline,
    /// Import already exists in imports block
    AlreadyImported,
    /// backends.kdl doesn't exist
    FileNotFound,
    /// No imports { } block found
    NoImportsBlock,
}

/// Initialize a new backend configuration file
///
/// Flow:
/// 1. Show "fetching [name] from nixval/declarch-packages"
/// 2. If not found → error
/// 3. Prompt "Are you sure you want this [name] being adopted? Y/n"
/// 4. If no → cancel
/// 5. If yes → write file
/// 6. Prompt "Need to import this automatically to backends.kdl? Y/n"
/// 7. If yes → add import, show "Backend 'name' adopted"
/// 8. If no → show "Backend 'name' fetched. please import it to use it"
pub fn init_backend(backend_name: &str, force: bool) -> Result<()> {
    let root_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    // STEP 1: Auto-initialize root if not exists
    if !config_file.exists() {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        fs::create_dir_all(&root_dir)?;
        let template = utils::templates::default_host(&hostname);
        fs::write(&config_file, template)?;
        output::success(&format!("Created config file: {}", config_file.display()));
        let _state = state::io::init_state(hostname)?;
    }

    // Sanitize backend name
    let sanitized_name: String = backend_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if sanitized_name.is_empty() {
        return Err(DeclarchError::Other(
            "Invalid backend name. Use alphanumeric characters, hyphens, or underscores.".to_string()
        ));
    }

    // STEP 2: Check if official backend
    if OFFICIAL_BACKENDS.contains(&sanitized_name.as_str()) {
        // Official backends are already embedded in backends.kdl - nothing to do
        output::info(&format!("Backend '{}' is already built-in.", sanitized_name));
        return Ok(());
    }

    // STEP 3: Create backends directory
    let backends_dir = root_dir.join("backends");
    if !backends_dir.exists() {
        fs::create_dir_all(&backends_dir)?;
    }

    // STEP 4: Show fetching message and fetch backend content
    println!("fetching '{}' from nixval/declarch-packages", sanitized_name);
    
    let backend_content = match remote::fetch_backend_content(&sanitized_name) {
        Ok(content) => content,
        Err(_) => {
            return Err(DeclarchError::Other(
                "not found, please check 'declarch init --list backends'".to_string()
            ));
        }
    };

    // STEP 5: Check if file already exists
    let backend_file = backends_dir.join(format!("{}.kdl", sanitized_name));
    if backend_file.exists() && !force {
        output::warning(&format!("Backend file already exists: {}", backend_file.display()));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    // STEP 6: Prompt for adoption confirmation (skip if force)
    if !force && !output::prompt_yes_no(&format!("Are you sure you want this '{}' being adopted", sanitized_name)) {
        output::info("Cancelled.");
        return Ok(());
    }

    // STEP 7: Write backend file
    fs::write(&backend_file, &backend_content)?;

    // STEP 8: Prompt for auto-import (skip if force)
    let want_import = if force {
        true
    } else {
        output::prompt_yes_no("Need to import this automatically to backends.kdl")
    };

    if want_import {
        let backends_kdl_path = root_dir.join("backends.kdl");
        let import_result = add_backend_import(&backends_kdl_path, &sanitized_name);
        
        match import_result {
            Ok(ImportResult::Added) => {
                println!("Backend '{}' adopted.", sanitized_name);
            }
            Ok(ImportResult::AlreadyImported) => {
                // Import already exists
                println!("Backend '{}' adopted.", sanitized_name);
            }
            Ok(ImportResult::AlreadyExistsInline) => {
                // Backend defined inline (built-in)
                println!("Backend '{}' adopted.", sanitized_name);
            }
            Ok(ImportResult::FileNotFound) => {
                output::warning("not automatically imported, please import it manually");
            }
            Ok(ImportResult::NoImportsBlock) => {
                output::warning("not automatically imported, please import it manually");
            }
            Err(e) => {
                // Error during import
                output::warning(&format!("not automatically imported: {}", e));
            }
        }
    } else {
        println!("Backend '{}' fetched. please import it to use it", sanitized_name);
    }

    Ok(())
}

/// Add backend import to backends.kdl imports block
pub fn add_backend_import(backends_kdl_path: &Path, backend_name: &str) -> Result<ImportResult> {
    // Check if backends.kdl exists
    if !backends_kdl_path.exists() {
        return Ok(ImportResult::FileNotFound);
    }
    
    let content = fs::read_to_string(backends_kdl_path)?;
    let import_path = format!("backends/{}.kdl", backend_name);
    
    // Check if backend already defined inline (default backends: aur, pacman, flatpak)
    let backend_pattern = format!(r#"(?m)^backend\s+"{}""#, regex::escape(backend_name));
    if Regex::new(&backend_pattern).map(|re| re.is_match(&content)).unwrap_or(false) {
        return Ok(ImportResult::AlreadyExistsInline);
    }
    
    // Check if already imported as an active import (not commented)
    // Look for the exact import line pattern
    let active_import_pattern = format!(r#"(?m)^\s+"{}""#, regex::escape(&import_path));
    if Regex::new(&active_import_pattern).map(|re| re.is_match(&content)).unwrap_or(false) {
        return Ok(ImportResult::AlreadyImported);
    }
    
    // Find imports block using regex that handles various formats:
    // - imports {
    // - imports{ 
    // - imports  {
    // And handles position anywhere in file (start, middle, end)
    let imports_re = Regex::new(r#"(?m)^(\s*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;
    
    if !imports_re.is_match(&content) {
        // No imports block found
        return Ok(ImportResult::NoImportsBlock);
    }
    
    // Format import line with proper indentation (4 spaces)
    let import_line = format!(r#"    "{}""#, import_path);
    
    // Find the imports block and add the import line
    let new_content = imports_re.replace(&content, |caps: &regex::Captures| {
        format!("{}\n{}", &caps[0], import_line)
    }).to_string();
    
    fs::write(backends_kdl_path, new_content)?;
    Ok(ImportResult::Added)
}

/// Remove backend import from backends.kdl (cleanup when removing backend)
#[allow(dead_code)]
pub fn remove_backend_import(backends_kdl_path: &Path, backend_name: &str) -> Result<()> {
    if !backends_kdl_path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(backends_kdl_path)?;
    let import_pattern = format!(r#"(?m)^\s+"backends/{}\.kdl"\s*$"#, regex::escape(backend_name));
    
    let re = Regex::new(&import_pattern)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;
    
    let new_content = re.replace_all(&content, "").to_string();
    fs::write(backends_kdl_path, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_backend_import_success() {
        // Test with imports block in the middle of file
        let content = r#"// Backend Aggregator

backend "aur" {
    binary "paru"
}

imports {
    // Custom backends
}

// Footer comment
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "npm").unwrap();
        assert_eq!(result, ImportResult::Added);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("backends/npm.kdl"));
        assert!(new_content.contains("imports {"));
    }

    #[test]
    fn test_add_backend_import_at_start() {
        // Test with imports block at start of file
        let content = r#"imports {
    // empty
}

backend "aur" {
    binary "paru"
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "cargo").unwrap();
        assert_eq!(result, ImportResult::Added);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("backends/cargo.kdl"));
    }

    #[test]
    fn test_add_backend_import_at_end() {
        // Test with imports block at end of file
        let content = r#"backend "aur" {
    binary "paru"
}

imports {
    // empty
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "pip").unwrap();
        assert_eq!(result, ImportResult::Added);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("backends/pip.kdl"));
    }

    #[test]
    fn test_add_backend_import_no_imports_block() {
        // Test when no imports block exists
        let content = r#"backend "aur" {
    binary "paru"
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "npm").unwrap();
        assert_eq!(result, ImportResult::NoImportsBlock);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(!new_content.contains("backends/npm.kdl")); // Should not be modified
    }

    #[test]
    fn test_add_backend_import_file_not_exist() {
        // Test when backends.kdl doesn't exist
        let non_existent_path = PathBuf::from("/tmp/non_existent_backends.kdl");
        
        let result = add_backend_import(&non_existent_path, "npm").unwrap();
        assert_eq!(result, ImportResult::FileNotFound);
    }

    #[test]
    fn test_add_backend_import_already_exists() {
        // Test when import already exists
        let content = r#"imports {
    "backends/npm.kdl"
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "npm").unwrap();
        assert_eq!(result, ImportResult::AlreadyImported);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        // Should not add duplicate
        let count = new_content.matches("backends/npm.kdl").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_backend_import_inline_backend_exists() {
        // Test when backend is already defined inline
        let content = r#"backend "npm" {
    binary "npm"
}

imports {
    // empty
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "npm").unwrap();
        assert_eq!(result, ImportResult::AlreadyExistsInline);
    }

    #[test]
    fn test_add_backend_import_various_whitespace() {
        // Test with various whitespace formats
        let content = r#"imports{
}

backend "aur" {
    binary "paru"
}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "cargo").unwrap();
        assert_eq!(result, ImportResult::Added);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("backends/cargo.kdl"));
    }

    #[test]
    fn test_add_backend_import_with_indentation() {
        // Test with indented imports block
        let content = r#"backend "aur" {
    binary "paru"
}

    imports {
        // Custom backends
    }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let result = add_backend_import(temp_file.path(), "pip").unwrap();
        assert_eq!(result, ImportResult::Added);
        
        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("backends/pip.kdl"));
    }
}

/// Generate default backends.kdl content
pub fn default_backends_kdl() -> &'static str {
    r#"// Backend Aggregator
// 
// Official backends (aur, pacman, flatpak) are embedded and ready to use.
// Custom backends can be added via 'declarch init --backend <name>'
//
// To disable: Comment out the backend block

// =============================================================================
// AUR Helper (Arch Linux)
// Fallback chain: paru → yay → pacman
// =============================================================================
backend "aur" {
    meta {
        title "AUR Helper"
        description "Arch User Repository with automatic fallback"
        maintained "declarch"
        tags "package-manager" "aur" "arch"
        platforms "arch"
        requires "paru" "yay" "pacman"
    }
    
    binary "paru" "yay"
    
    list "{binary} -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "{binary} -S --needed {packages}"
    remove "{binary} -R {packages}"
    
    search "{binary} -Ss {query}" {
        format "regex"
        regex "(?m)^(\\S+)\\s+.*\\n\\s+(.*)$"
        name_group 1
        desc_group 2
    }
    
    fallback "pacman"
}

// =============================================================================
// Pacman (Arch Linux native)
// =============================================================================
backend "pacman" {
    meta {
        title "Pacman"
        description "Arch Linux native package manager"
        maintained "declarch"
        tags "package-manager" "arch" "native"
        platforms "arch"
        requires "pacman"
    }
    
    binary "pacman"
    
    list "pacman -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "pacman -S --needed {packages}"
    remove "pacman -R {packages}"
    
    needs_sudo "true"
}

// =============================================================================
// Flatpak (Universal Linux apps)
// =============================================================================
backend "flatpak" {
    meta {
        title "Flatpak"
        description "Universal Linux application sandboxing"
        maintained "declarch"
        tags "package-manager" "flatpak" "sandbox" "universal"
        platforms "linux"
        requires "flatpak"
    }
    
    binary "flatpak"
    
    list "flatpak list --app --columns=application,version" {
        format "tsv"
        name_col 0
        version_col 1
    }
    
    install "flatpak install flathub {packages}"
    remove "flatpak uninstall {packages}"
    
    search "flatpak search {query}" {
        format "tsv"
        name_col 2
        desc_col 1
    }
    
    noconfirm "-y"
    needs_sudo "false"
}

// =============================================================================
// NPM (Node.js packages)
// =============================================================================
backend "npm" {
    meta {
        title "NPM"
        description "Node.js package manager"
        maintained "declarch"
        tags "package-manager" "nodejs" "javascript"
        platforms "linux" "macos"
        requires "npm"
    }
    
    binary "npm"
    
    list "npm list -g --json" {
        format "json"
        json_path "dependencies"
        name_key "name"
        version_key "version"
    }
    
    install "npm install -g --silent {packages}"
    remove "npm uninstall -g --silent {packages}"
    
    needs_sudo "false"
}

// =============================================================================
// Custom Backends
// Use 'declarch init --backend <name>' to add custom backends
// =============================================================================
imports {
    // Custom backend imports will be added here
}
"#
}
