//! Backend initialization logic
//!
//! Handles the `declarch init --backend <name>` command flow:
//! 1. Fetch backend definition from remote registry
//! 2. Validate and sanitize backend name
//! 3. Write backend file to `backends/<name>.kdl`
//! 4. Add import to `backends.kdl`

use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::{paths, remote};
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
/// 3. Validate KDL (warning if invalid, can bypass with --force)
/// 4. Prompt "Are you sure you want this [name] being adopted? Y/n"
/// 5. If no → cancel
/// 6. If yes → write file
/// 7. Prompt "Need to import this automatically to backends.kdl? Y/n"
/// 8. If yes → add import, show "Backend 'name' adopted"
/// 9. If no → show "Backend 'name' fetched. please import it to use it"
pub fn init_backend(backend_name: &str, force: bool) -> Result<()> {
    let root_dir = paths::config_dir()?;

    // STEP 1: Ensure declarch environment exists (auto-init if needed)
    super::root::ensure_environment()?;

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

    // STEP 4b: Validate KDL (warning only, can bypass with --force)
    if let Err(e) = super::validate_kdl(&backend_content, &format!("backend '{}'", sanitized_name)) {
        if !force {
            output::warning(&format!("{}", e));
            output::info("The backend may be malformed or incompatible with your declarch version.");
            output::info("You can still adopt it with --force, then edit the file manually.");
            
            if !output::prompt_yes_no("Continue with potentially invalid backend") {
                output::info("Cancelled. You can try a different backend or use --force to override.");
                return Ok(());
            }
        }
    }

    // STEP 4c: Parse and display meta information
    if let Ok(meta) = extract_backend_meta(&backend_content) {
        println!();
        if !meta.title.is_empty() && meta.title != "-" {
            println!("  Title:       {}", meta.title);
        }
        if !meta.description.is_empty() && meta.description != "-" {
            println!("  Description: {}", meta.description);
        }
        if !meta.maintained.is_empty() && meta.maintained != "-" {
            println!("  Maintained:  {}", meta.maintained);
        }
        if !meta.homepage.is_empty() && meta.homepage != "-" {
            println!("  Homepage:    {}", meta.homepage);
        }
        if !meta.platforms.is_empty() {
            println!("  Platforms:   {}", meta.platforms.join(", "));
        }
        if !meta.requires.is_empty() && meta.requires != "-" {
            println!("  Requires:    {}", meta.requires);
        }
        println!();
    }

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

    // STEP 7: Write backend file (KDL already validated or bypassed)
    fs::write(&backend_file, &backend_content)?;

    // STEP 8: Auto-import logic
    // Check if backends.kdl exists
    let backends_kdl_path = root_dir.join("backends.kdl");
    let declarch_kdl_path = root_dir.join("declarch.kdl");
    
    if backends_kdl_path.exists() {
        // Standard flow: import to backends.kdl
        let want_import = if force {
            true
        } else {
            output::prompt_yes_no("Import this backend to backends.kdl")
        };

        if want_import {
            let import_result = add_backend_import(&backends_kdl_path, &sanitized_name);
            
            match import_result {
                Ok(ImportResult::Added) |
                Ok(ImportResult::AlreadyImported) |
                Ok(ImportResult::AlreadyExistsInline) => {
                    println!("Backend '{}' adopted.", sanitized_name);
                }
                Ok(ImportResult::FileNotFound) |
                Ok(ImportResult::NoImportsBlock) => {
                    output::warning("Could not auto-import. Please add to backends.kdl manually:");
                    output::info(&format!(r#"    "backends/{}.kdl""#, sanitized_name));
                }
                Err(e) => {
                    output::warning(&format!("Could not auto-import: {}", e));
                }
            }
        } else {
            println!("Backend '{}' fetched. Add to backends.kdl to use:", sanitized_name);
            output::info(&format!(r#"    "backends/{}.kdl""#, sanitized_name));
        }
    } else {
        // backends.kdl doesn't exist: import directly to declarch.kdl
        let import_result = add_backend_to_declarch(&declarch_kdl_path, &sanitized_name);
        
        match import_result {
            Ok(true) => {
                println!("Backend '{}' adopted (added to declarch.kdl).", sanitized_name);
            }
            Ok(false) => {
                // backends {} block not found, manual import needed
                println!("Backend '{}' fetched. Add to declarch.kdl to use:", sanitized_name);
                output::info(&format!("backends {{\"backends/{}.kdl\"}}", sanitized_name));
            }
            Err(e) => {
                output::warning(&format!("Could not auto-import: {}", e));
                output::info(&format!("Add manually to declarch.kdl: backends {{\"backends/{}.kdl\"}}", sanitized_name));
            }
        }
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

/// Add backend import directly to declarch.kdl (fallback when backends.kdl doesn't exist)
/// 
/// Returns:
/// - Ok(true): Successfully added
/// - Ok(false): backends {} block not found, manual import needed
/// - Err: Error during file operation
pub fn add_backend_to_declarch(declarch_kdl_path: &Path, backend_name: &str) -> Result<bool> {
    if !declarch_kdl_path.exists() {
        return Err(DeclarchError::Other(
            format!("declarch.kdl not found at {}", declarch_kdl_path.display())
        ));
    }
    
    let content = fs::read_to_string(declarch_kdl_path)?;
    let import_path = format!("backends/{}.kdl", backend_name);
    
    // Check if already imported
    let existing_pattern = format!(r#""{}""#, regex::escape(&import_path));
    if Regex::new(&existing_pattern).map(|re| re.is_match(&content)).unwrap_or(false) {
        return Ok(true); // Already exists
    }
    
    // Look for backends { ... } block
    let backends_re = Regex::new(r#"(?m)^(	*backends)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex: {}", e)))?;
    
    if !backends_re.is_match(&content) {
        // No backends block found - user needs to add manually
        return Ok(false);
    }
    
    // Find backends block and add import line
    // Pattern matches: backends { or backends "existing" {
    let backends_block_re = Regex::new(r#"(?m)^(	*backends(?:	*"[^"]*")?	*
?)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex: {}", e)))?;
    
    let import_line = format!(r#"    "{}""#, import_path);
    
    let new_content = backends_block_re.replace(&content, |caps: &regex::Captures| {
        format!("{}\n{}", &caps[0], import_line)
    }).to_string();
    
    fs::write(declarch_kdl_path, new_content)?;
    Ok(true)
}

/// Backend meta information extracted from KDL
#[derive(Debug, Default)]
pub struct BackendMeta {
    pub title: String,
    pub description: String,
    pub maintained: String,
    pub homepage: String,
    pub platforms: Vec<String>,
    pub requires: String,
}

/// Extract meta information from backend KDL content
pub fn extract_backend_meta(content: &str) -> Result<BackendMeta> {
    let doc = kdl::KdlDocument::parse(content)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend KDL: {}", e)))?;
    
    let mut meta = BackendMeta::default();
    
    // Find the backend node
    for node in doc.nodes() {
        if node.name().value() == "backend" {
            // Look for meta block in children
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    if child.name().value() == "meta" {
                        if let Some(meta_children) = child.children() {
                            for meta_node in meta_children.nodes() {
                                let name = meta_node.name().value();
                                match name {
                                    "title" => {
                                        meta.title = meta_node.entries().first()
                                            .and_then(|e| e.value().as_string())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                    "description" => {
                                        meta.description = meta_node.entries().first()
                                            .and_then(|e| e.value().as_string())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                    "maintained" => {
                                        meta.maintained = meta_node.entries().first()
                                            .and_then(|e| e.value().as_string())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                    "homepage" => {
                                        meta.homepage = meta_node.entries().first()
                                            .and_then(|e| e.value().as_string())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                    "requires" => {
                                        meta.requires = meta_node.entries().first()
                                            .and_then(|e| e.value().as_string())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                    "platforms" => {
                                        for entry in meta_node.entries() {
                                            if let Some(platform) = entry.value().as_string() {
                                                meta.platforms.push(platform.to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        break;
                    }
                }
            }
            break;
        }
    }
    
    Ok(meta)
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
    r#"// ============================================================================
// BACKENDS.KDL - Declarch Backend Configuration
// ============================================================================
//
// This file defines all package manager backends for declarch.
// Each backend describes how to interact with a package manager.
//
// LOCATION: ~/.config/declarch/backends.kdl
//
// ============================================================================
// COMMAND REFERENCE
// ============================================================================
//
// Required commands:
//   install "cmd {packages}"     - Install packages ({packages} = space-separated list)
//
// Optional commands (omit or use "-" to disable without warning):
//   list "cmd" { ... }          - List installed packages (omit = install-only backend)
//   remove "cmd {packages}"      - Remove packages (omit = cannot remove via declarch)
//   update "cmd"                 - Update package index (use "-" if not applicable)
//   upgrade "cmd"                - Upgrade packages to latest version
//   cache_clean "cmd"            - Clean package cache
//   search "cmd {query}" { ... } - Search remote repositories
//   search_local "cmd {query}" { ... } - Search locally installed packages
//
// Sentinel value "-" (dash):
//   Use "-" to explicitly disable a command without getting warnings.
//   Example: update "-"  (NPM doesn't have an update index command)
//
// Placeholders:
//   {binary}  - Replaced with the actual binary name (respects fallback chain)
//   {packages} - Space-separated package list (for install/remove)
//   {query}   - Search term (for search/search_local)
//
// ============================================================================
// OUTPUT FORMATS
// ============================================================================
//
// For list, search, and search_local commands, specify how to parse output:
//
//   format "whitespace"          - Space-separated columns (default)
//     name_col 0                 - Column index for package name
//     version_col 1              - Column index for version (optional)
//
//   format "tsv"                 - Tab-separated columns
//     name_col 0
//     version_col 1
//
//   format "json"                - JSON output
//     json_path "dependencies"   - Path to array in JSON (dot notation)
//     name_key "name"            - Key for package name
//     version_key "version"      - Key for version (optional)
//
//   format "regex"               - Regex pattern matching
//     regex "pattern"            - Regex with capture groups
//     name_group 1               - Capture group for package name
//     version_group 2            - Capture group for version (optional)
//
// ============================================================================

// =============================================================================
// AUR Helper (Arch Linux)
// =============================================================================
// Supports: paru, yay (with pacman fallback)
// Features: Full support (install, remove, update, upgrade, cache clean, search)
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
    
    // Try paru first, then yay
    binary "paru" "yay"
    
    // List installed packages
    list "{binary} -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    // Package operations
    install "{binary} -S --needed {packages}"
    remove "{binary} -R {packages}"
    
    // System operations
    update "{binary} -Sy"              // Update package database
    upgrade "{binary} -Syu"            // Upgrade all packages
    cache_clean "{binary} -Sc --noconfirm"  // Clean package cache
    
    // Search remote repositories
    search "{binary} -Ss {query}" {
        format "regex"
        regex "(?m)^(\\S+)\\s+.*\\n\\s+(.*)$"
        name_group 1
        desc_group 2
    }
    
    // Search locally installed packages
    search_local "{binary} -Q {query}" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    // Fallback to pacman if no AUR helper available
    fallback "pacman"
}

// =============================================================================
// Pacman (Arch Linux native)
// =============================================================================
// Features: Official repo only (no AUR support)
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
    
    list "{binary} -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "{binary} -S --needed {packages}"
    remove "{binary} -R {packages}"
    
    update "{binary} -Sy"
    upgrade "{binary} -Syu"
    cache_clean "{binary} -Sc --noconfirm"
    
    search_local "{binary} -Q {query}" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    needs_sudo "true"
}

// =============================================================================
// Flatpak (Universal Linux apps)
// =============================================================================
// Features: Sandbox apps from Flathub
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
    
    list "{binary} list --app --columns=application,version" {
        format "tsv"
        name_col 0
        version_col 1
    }
    
    install "{binary} install flathub {packages}"
    remove "{binary} uninstall {packages}"
    
    update "{binary} update --appstream"
    upgrade "{binary} update -y"
    cache_clean "{binary} uninstall --unused -y"
    
    search "{binary} search {query}" {
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
// Note: NPM doesn't have a traditional "update" command (no package index).
// Use "-" sentinel to explicitly disable without warning.
//
// IMPORTANT: npm list -g --json outputs:
//   {"dependencies": {"package-name": {"version": "1.0"}}}
// The keys ARE the package names (not a "name" field inside).
// Use format "json_object_keys" for this structure.
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
    
    list "{binary} list -g --json" {
        format "json_object_keys"
        json_path "dependencies"
        version_key "version"
    }
    
    install "{binary} install -g --silent {packages}"
    remove "{binary} uninstall -g --silent {packages}"
    
    // Sentinel "-" = explicitly disabled (NPM has no package index update)
    update "-"
    upgrade "{binary} update -g"
    cache_clean "{binary} cache clean --force"
    
    search "{binary} search {query} --json" {
        format "npm_json"
        name_key "name"
        version_key "version"
        desc_key "description"
    }
    
    // Search locally installed packages
    search_local "{binary} list -g {query} --json" {
        format "json_object_keys"
        json_path "dependencies"
        version_key "version"
    }
    
    needs_sudo "false"
}

// =============================================================================
// EXAMPLE: APT (Debian/Ubuntu) - Uncomment to use
// =============================================================================
// backend "apt" {
//     binary "apt"
//     
//     list "apt list --installed" {
//         format "regex"
//         regex {
//             pattern "^(\\S+)/\\S+\\s+\\S+\\s+\\[installed.*\]"
//             name_group 1
//         }
//     }
//     
//     install "apt install {packages}"
//     remove "apt remove {packages}"
//     update "apt update"
//     upgrade "apt upgrade -y"
//     cache_clean "apt clean"
//     
//     noconfirm "-y"
//     needs_sudo "true"
// }

// =============================================================================
// EXAMPLE: Install-only backend (no list/remove)
// =============================================================================
// For tools that can install but declarch can't track/uninstall:
//
// backend "mycustom" {
//     binary "mycustom"
//     install "mycustom install {packages}"
//     // Omit list/remove = install-only backend
// }

// =============================================================================
// Custom Backends
// Use 'declarch init --backend <name>' to fetch from remote registry
// =============================================================================
imports {
    // Custom backend imports will be added here automatically
    // Example: "backends/cargo.kdl"
}
"#
}
