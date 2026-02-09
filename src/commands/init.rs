use crate::config::kdl::parse_kdl_content;
use crate::constants::{CONFIG_EXTENSION, CONFIG_FILE_NAME};
use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use crate::utils::{self, paths, remote};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Default backends.kdl content with embedded official backends
const DEFAULT_BACKENDS_KDL: &str = r#"// Backend Aggregator
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
"#;

#[derive(Debug)]
pub struct InitOptions {
    pub path: Option<String>,
    pub host: Option<String>,
    pub backends: Vec<String>,
    pub force: bool,
    pub yes: bool,
    pub local: bool,
}

pub fn run(options: InitOptions) -> Result<()> {
    // CASE A: MODULE INITIALIZATION
    if let Some(target_path) = options.path {
        return init_module(&target_path, options.force, options.yes, options.local);
    }

    // CASE A2: BACKEND INITIALIZATION (supports multiple)
    if !options.backends.is_empty() {
        let force = options.force || options.yes;
        let total = options.backends.len();
        
        // Ensure root config exists first
        let config_file = paths::config_file()?;
        if !config_file.exists() {
            init_root(options.host.clone(), force)?;
        }
        
        if total > 1 {
            output::header(&format!("Initializing {} backends", total));
        }
        
        for (i, backend_name) in options.backends.iter().enumerate() {
            if total > 1 {
                println!();
                output::info(&format!("[{}/{}] Initializing '{}'", i + 1, total, backend_name));
            }
            init_backend(backend_name, force)?;
        }
        return Ok(());
    }

    // CASE B: ROOT INITIALIZATION
    init_root(options.host, options.force)
}

/// Initialize root configuration
fn init_root(host: Option<String>, force: bool) -> Result<()> {
    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    if config_file.exists() && !force {
        println!("Declarch is already initialized.");
        println!("  {}", config_dir.display());
        return Ok(());
    }

    let hostname = host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    // Create directory structure
    let backends_dir = config_dir.join("backends");
    let modules_dir = config_dir.join("modules");
    
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&backends_dir)?;
    fs::create_dir_all(&modules_dir)?;

    // Create default files
    let backends_kdl_path = config_dir.join("backends.kdl");
    if !backends_kdl_path.exists() {
        fs::write(&backends_kdl_path, DEFAULT_BACKENDS_KDL)?;
    }

    let template = utils::templates::default_host(&hostname);
    fs::write(&config_file, template)?;

    let base_module_path = modules_dir.join(format!("base.{}", CONFIG_EXTENSION));
    if !base_module_path.exists() {
        let base_template = utils::templates::get_template_by_name("base")
            .unwrap_or_else(|| utils::templates::default_module("base"));
        fs::write(&base_module_path, base_template)?;
    }

    let _state = state::io::init_state(hostname.clone())?;
    
    // Simple, clean output
    println!("Created declarch directory:");
    println!("  {}", config_dir.display());
    println!("Initializing declarch for host: {}", hostname);

    Ok(())
}

/// Official backends that are embedded in backends.kdl
const OFFICIAL_BACKENDS: &[&str] = &["aur", "pacman", "flatpak"];

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
fn init_backend(backend_name: &str, force: bool) -> Result<()> {
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

/// Result of attempting to add backend import
#[derive(Debug, PartialEq)]
enum ImportResult {
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

/// Add backend import to backends.kdl imports block
fn add_backend_import(backends_kdl_path: &Path, backend_name: &str) -> Result<ImportResult> {
    // Check if backends.kdl exists
    if !backends_kdl_path.exists() {
        return Ok(ImportResult::FileNotFound);
    }
    
    let content = fs::read_to_string(backends_kdl_path)?;
    let import_path = format!("backends/{}.kdl", backend_name);
    
    // Check if backend already defined inline (default backends: aur, pacman, flatpak)
    let backend_pattern = format!(r#"(?m)^backend\s+"{}""#, regex::escape(backend_name));
    if regex::Regex::new(&backend_pattern).map(|re| re.is_match(&content)).unwrap_or(false) {
        return Ok(ImportResult::AlreadyExistsInline);
    }
    
    // Check if already imported as an active import (not commented)
    // Look for the exact import line pattern
    let active_import_pattern = format!(r#"(?m)^\s+"{}""#, regex::escape(&import_path));
    if regex::Regex::new(&active_import_pattern).map(|re| re.is_match(&content)).unwrap_or(false) {
        return Ok(ImportResult::AlreadyImported);
    }
    
    // Find imports block using regex that handles various formats:
    // - imports {
    // - imports{ 
    // - imports  {
    // And handles position anywhere in file (start, middle, end)
    let imports_re = regex::Regex::new(r#"(?m)^(\s*imports\s*\{)"#)
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
fn remove_backend_import(backends_kdl_path: &Path, backend_name: &str) -> Result<()> {
    if !backends_kdl_path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(backends_kdl_path)?;
    let import_pattern = format!(r#"(?m)^\s+"backends/{}\.kdl"\s*$"#, regex::escape(backend_name));
    
    let re = regex::Regex::new(&import_pattern)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;
    
    let new_content = re.replace_all(&content, "").to_string();
    fs::write(backends_kdl_path, new_content)?;
    Ok(())
}

// Note: Template generation is kept for future local backend creation feature
#[allow(dead_code)]
fn generate_backend_template(name: &str) -> String {
    match name {
        "aur" => generate_aur_backend_template(),
        "pacman" => generate_pacman_backend_template(),
        "flatpak" => generate_flatpak_backend_template(),
        _ => generate_generic_backend_template(name),
    }
}

#[allow(dead_code)]
fn generate_aur_backend_template() -> String {
    r#"// aur - AUR Helper with Fallback Chain
// 
// This backend provides a complete fallback chain:
//   aur (alias) → paru → yay → pacman
//
// Behavior:
//   1. If 'paru' is installed, use it (best AUR helper)
//   2. Else if 'yay' is installed, use it (alternative AUR helper)
//   3. Else if 'pacman' backend exists, fallback to it (official repos only)
//
// Installation:
//   paru:  pacman -S paru      (or from AUR: paru-bin)
//   yay:   pacman -S yay       (or from AUR: yay-bin)

backend "aur" {
    meta {
        title "AUR Helper"
        description "Arch User Repository with automatic fallback chain"
        maintained "declarch"
        tags "package-manager" "aur" "arch" "fallback"
        platforms "arch"
        requires "paru" "yay" "pacman"
    }
    
    // Binary fallback chain: try paru first, then yay
    binary "paru" "yay"
    
    // List all installed packages (official repo + AUR)
    // Output format: "package-name version"
    list "{binary} -Q" {
        format whitespace
        name_col 0
        version_col 1
    }
    
    // Install packages (supports both AUR and official repos)
    install "{binary} -S --needed {packages}"
    
    // Remove packages
    remove "{binary} -R {packages}"
    
    // Search packages in AUR and repos
    search "{binary} -Ss {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
    
    // Query package info (for dependencies)
    // query "{binary} -Qi {package}"
    
    // If no AUR helper available, fallback to pacman
    fallback "pacman"
}
"#.to_string()
}

#[allow(dead_code)]
fn generate_pacman_backend_template() -> String {
    r#"// pacman - Arch Linux Native Package Manager
//
// This is the ultimate fallback when no AUR helper is available.
// It only supports official repositories, not AUR.

backend "pacman" {
    meta {
        title "Pacman"
        description "Arch Linux native package manager (official repos only)"
        maintained "declarch"
        tags "package-manager" "arch" "native" "official"
        platforms "arch"
        requires "pacman"
    }
    
    binary "pacman"
    
    list "pacman -Q" {
        format whitespace
        name_col 0
        version_col 1
    }
    
    install "pacman -S --needed {packages}"
    remove "pacman -R {packages}"
    
    search "pacman -Ss {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
    
    needs_sudo true
}
"#.to_string()
}

#[allow(dead_code)]
fn generate_flatpak_backend_template() -> String {
    r#"// flatpak - Universal Linux Application Sandboxing
//
// Flatpak runs applications in isolated sandboxes with their own dependencies.
// Applications are identified by reverse DNS: com.vendor.AppName

backend "flatpak" {
    meta {
        title "Flatpak"
        description "Universal Linux application sandboxing"
        maintained "declarch"
        tags "package-manager" "flatpak" "sandbox" "universal" "gui"
        platforms "linux"
        requires "flatpak"
    }
    
    binary "flatpak"
    
    // List installed applications
    // Output: Application ID    Version    Branch    Installation
    list "flatpak list --app --columns=application,version" {
        format tsv
        name_col 0
        version_col 1
    }
    
    // Install from flathub (default remote)
    install "flatpak install flathub {packages}"
    
    // Remove applications
    remove "flatpak uninstall {packages}"
    
    // Search flathub
    search "flatpak search {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
    
    // Auto-confirm operations
    noconfirm "-y"
    
    // Flatpak doesn't need sudo for user installations
    needs_sudo false
}
"#.to_string()
}

#[allow(dead_code)]
fn generate_generic_backend_template(name: &str) -> String {
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    format!(
        r#"// {name} - Custom Backend Configuration
// 
// This is a template for the '{name}' package manager.
// Customize the commands below to match your package manager's syntax.
//
// Common placeholders:
//   {{packages}}  - Space-separated package names
//   {{query}}     - Search query (for search command)
//
// Output formats:
//   - whitespace: Space-separated columns (default)
//   - tsv:        Tab-separated values
//   - json:       JSON output with configurable path
//   - regex:      Regular expression pattern matching

backend "{name}" {{
    meta {{
        title "{name}"
        description "Custom {name} backend configuration"
        maintained "user"
        tags "package-manager" "{name}"
        created "{date}"
        platforms "linux"
        requires "{name}"
    }}
    
    // Binary to use (can specify multiple for fallback)
    binary "{name}"
    
    // Command to list installed packages
    //
    // Example outputs and formats:
    // - "package version"        → format whitespace, name_col 0, version_col 1
    // - "package\tversion"       → format tsv, name_col 0, version_col 1
    // - '{{"packages":[{{"name":"x"}}]}}' → format json, json {{ path "packages" name_key "name" }}
    list "{name} list" {{
        format whitespace
        name_col 0
        // version_col 1  // Uncomment if output includes version
    }}
    
    // Install command
    // {{packages}} will be replaced with space-separated package names
    install "{name} install {{packages}}"
    
    // Remove/uninstall command
    remove "{name} remove {{packages}}"
    
    // Search command (optional but recommended)
    // Uncomment and customize if your package manager supports search
    // search "{name} search {{query}}" {{
    //     format whitespace
    //     name_col 0
    //     desc_col 1
    // }}
    
    // Auto-confirmation flag (optional)
    // Uncomment if your package manager has a "yes to all" flag
    // noconfirm "-y"
    
    // Whether this backend requires sudo (default: false)
    // Uncomment if install/remove need root privileges
    // needs_sudo true
    
    // Fallback backend (optional)
    // If this backend's binary is not found, use another backend instead
    // fallback "alternative-backend"
}}
"#,
        name = name,
        date = current_date
    )
}

fn init_module(target_path: &str, force: bool, yes: bool, local: bool) -> Result<()> {
    output::header("Importing Module");

    let root_dir = paths::config_dir()?;

    // Auto-initialize root if not exists
    if !root_dir.exists() {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        // Create root directory
        fs::create_dir_all(&root_dir)?;

        // Create default config
        let config_file = paths::config_file()?;
        let template = utils::templates::default_host(&hostname);
        fs::write(&config_file, template)?;
        output::success(&format!("Created config file: {}", config_file.display()));

        // Initialize state
        let _state = state::io::init_state(hostname.clone())?;
    }

    // 1. Resolve Path - preserve directory structure
    let mut path_buf = PathBuf::from(target_path);
    if path_buf.extension().is_none() {
        path_buf.set_extension(CONFIG_EXTENSION);
    }

    // Always prepend "modules/" to keep structure
    let modules_path = PathBuf::from("modules").join(&path_buf);
    let full_path = root_dir.join(&modules_path);

    // 2. Check Existence
    if full_path.exists() && !force {
        output::warning(&format!("Module already exists: {}", full_path.display()));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    // Create parent directories
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 3. Select Template - use full target_path for remote fetch
    let slug = path_buf
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Check if user requested a namespaced module (e.g., "category/name")
    let is_registry_path = target_path.contains('/') || target_path.contains('\\');

    // STRATEGY A: Hardcoded Template (Fastest, Offline)
    let content = if let Some(local_tmpl) = utils::templates::get_template_by_name(&slug) {
        local_tmpl
    }
    // STRATEGY B: Remote Registry (The "Marketplace") - skip if --local flag
    else if is_registry_path && !local {
        // User explicitly requested a registry module - fail if not found
        match remote::fetch_module_content(target_path) {
            Ok(remote_content) => remote_content,
            Err(e) => {
                return Err(DeclarchError::ConfigError(format!(
                    "Failed to fetch module '{}' from registry: {}\n\n\
                    Try one of these alternatives:\n\
                    1. List available modules:    declarch init --list modules\n\
                    2. Create local module:       declarch init --local {}\n\
                    3. Use simple name:           declarch init {}",
                    target_path, e, slug, slug
                )));
            }
        }
    }
    // STRATEGY C: Create new local module from template
    else {
        // User just wants to create a new module locally (or --local flag used)
        if local {
            output::info(&format!("Creating local module: {}", slug));
        } else {
            output::info(&format!("Creating new local module: {}", slug));
        }
        utils::templates::default_module(&slug)
    };

    // Display module meta information before proceeding
    display_module_meta(&content);

    // 4. Write File
    fs::write(&full_path, &content)?;

    // 5. AUTO INJECT IMPORT
    let root_config_path = paths::config_file()?;
    let import_path = modules_path.to_string_lossy().replace("\\", "/");
    inject_import_to_root(&root_config_path, &import_path, force, yes)?;

    output::success("Done");

    Ok(())
}

/// Extract and display meta information from KDL content
fn display_module_meta(content: &str) {
    // Try to parse the content and extract meta
    if let Ok(raw_config) = parse_kdl_content(content) {
        let meta = &raw_config.project_metadata;

        // Only display if we have some meta information
        let has_meta = meta.title.is_some()
            || meta.description.is_some()
            || meta.author.is_some()
            || meta.version.is_some()
            || !meta.tags.is_empty()
            || meta.url.is_some();

        if has_meta {
            output::separator();
            println!("{}", "Module Information:".bold().cyan());

            if let Some(title) = &meta.title {
                println!("  {}", title.bold());
                println!();
            }

            if let Some(description) = &meta.description {
                println!("  {}", description.dimmed());
                println!();
            }

            let mut details = Vec::new();

            if let Some(author) = &meta.author {
                details.push(format!("Author: {}", author.yellow()));
            }

            if let Some(version) = &meta.version {
                details.push(format!("Version: {}", version.green()));
            }

            if !meta.tags.is_empty() {
                details.push(format!("Tags: {}", meta.tags.join(", ").purple()));
            }

            if let Some(url) = &meta.url {
                details.push(format!("URL: {}", url.blue().underline()));
            }

            if !details.is_empty() {
                for detail in details {
                    println!("  {}", detail);
                }
            }

            output::separator();
        }
    }
}

/// Helper to inject the import statement into main config file using Regex
fn inject_import_to_root(config_path: &Path, import_path: &str, force: bool, yes: bool) -> Result<()> {
    let content = fs::read_to_string(config_path)?;

    // Pattern to insert: "path/to/module.{extension}"
    // We add quotes for safety.
    let import_line = format!("    {:?}", import_path);

    // 1. Check if it already exists (Simple check)
    // This catches both active imports and commented ones.
    // We assume if the string is there, the user knows about it.
    if content.contains(import_path) {
        output::info(&format!(
            "Module '{}' is already referenced in config.",
            import_path
        ));
        return Ok(());
    }

    // Prompt for consent unless force or yes is active
    if !force && !yes
        && !output::prompt_yes_no(&format!(
            "Add '{}' to imports in {}?",
            import_path, CONFIG_FILE_NAME
        ))
    {
        output::info("Skipping auto-import. You can add it manually.");
        return Ok(());
    }

    // 2. Regex Magic
    // Find: imports { (with flexible whitespace)
    let re = Regex::new(r#"(?m)^(.*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        // INJECT: Insert right after the opening brace
        // $1 references the captured "imports {" line
        // We append a newline and our import line
        re.replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        })
        .to_string()
    } else {
        // FALLBACK: Append new block if not found
        format!(
            "{}\n\nimports {{\n{}\n}}\n",
            content.trim_end(),
            import_line
        )
    };

    fs::write(config_path, new_content)?;

    Ok(())
}


/// List available backends from the registry
pub fn list_available_backends() -> Result<()> {
    output::header("Available Backends");
    
    // Static list of known backends in the registry
    // This could be fetched dynamically from the registry API in the future
    let backends = vec![
        ("apt", "Debian/Ubuntu package manager", vec!["debian", "ubuntu", "linux"]),
        ("aur", "Meta backend for AUR (paru→yay→pacman fallback)", vec!["arch", "linux", "meta"]),
        ("brew", "Homebrew for macOS/Linux", vec!["macos", "linux", "homebrew"]),
        ("bun", "Fast JavaScript toolkit", vec!["nodejs", "javascript", "fast"]),
        ("cargo", "Rust package manager", vec!["rust", "build-system"]),
        ("dnf", "Fedora/RHEL package manager", vec!["fedora", "rhel", "linux"]),
        ("flatpak", "Universal Linux apps", vec!["linux", "universal", "desktop"]),
        ("gem", "Ruby package manager", vec!["ruby", "gems"]),
        ("go", "Go package installer", vec!["go", "golang"]),
        ("nix", "Nix functional package manager", vec!["nix", "nixos", "functional"]),
        ("npm", "Node.js package manager", vec!["nodejs", "javascript"]),
        ("pacman", "Arch Linux native package manager", vec!["arch", "linux", "native"]),
        ("paru", "Feature-packed AUR helper", vec!["arch", "linux", "aur", "rust"]),
        ("pip", "Python package installer", vec!["python", "pypi"]),
        ("pnpm", "Fast, disk space efficient package manager", vec!["nodejs", "javascript", "fast"]),
        ("snap", "Universal packages by Canonical", vec!["linux", "universal", "canonical"]),
        ("soar", "Static binary package manager", vec!["linux", "static", "fast"]),
        ("yarn", "Fast, reliable dependency management", vec!["nodejs", "javascript"]),
        ("yay", "Yet Another Yogurt AUR helper", vec!["arch", "linux", "aur", "go"]),
    ];
    
    println!();
    const SYSTEM_PMS: &[&str] = &["apt", "aur", "dnf", "pacman", "paru", "yay", "flatpak", "snap", "nix", "soar", "brew"];
    const LANG_PMS: &[&str] = &["npm", "yarn", "pnpm", "bun", "pip", "cargo", "gem", "go"];
    
    println!("{}", "System Package Managers:".bold().cyan());
    for (name, desc, _) in &backends {
        if SYSTEM_PMS.contains(name) {
            println!("  {} - {}", name.bold(), desc.dimmed());
        }
    }
    
    println!();
    println!("{}", "Language-Specific Package Managers:".bold().cyan());
    for (name, desc, _) in &backends {
        if LANG_PMS.contains(name) {
            println!("  {} - {}", name.bold(), desc.dimmed());
        }
    }
    
    println!();
    output::info("To initialize a backend:");
    println!("  {}        Initialize single backend", "declarch init --backend <name>".green());
    println!("  {}  Initialize multiple backends", "declarch init --backend a,b,c".green());
    println!("  {}        Initialize with auto-yes", "declarch init --backend <name> -y".green());
    
    Ok(())
}

/// List available modules from the registry
pub fn list_available_modules() -> Result<()> {
    output::header("Available Modules");
    
    // Static list of known modules in the registry
    // This could be fetched dynamically from the registry API in the future
    let modules = vec![
        ("system/base", "Essential packages for any Linux system", vec!["base", "essential", "core"]),
        ("desktop/hyprland", "Hyprland Wayland compositor setup", vec!["desktop", "wayland", "hyprland"]),
        ("apps/flatpak-common", "Common GUI applications via Flatpak", vec!["apps", "gui", "flatpak"]),
        ("development/nodejs", "Node.js development environment", vec!["dev", "nodejs", "javascript"]),
        ("dev/rust", "Rust development tools", vec!["dev", "rust", "cargo"]),
        ("apps/multimedia", "Multimedia applications", vec!["apps", "media", "audio", "video"]),
    ];
    
    // Group by category
    let mut by_category: std::collections::HashMap<&str, Vec<&(&str, &str, Vec<&str>)>> = std::collections::HashMap::new();
    for m in &modules {
        let category = m.0.split('/').next().unwrap_or("other");
        by_category.entry(category).or_default().push(m);
    }
    
    println!();
    for (category, cat_modules) in by_category {
        println!("{}", format!("{}/", category).bold().cyan());
        for (name, desc, tags) in cat_modules {
            let module_name = name.split('/').nth(1).unwrap_or(name);
            println!("  {} - {} {}", 
                module_name.bold(), 
                desc.dimmed(),
                format!("[{}]", tags.join(", ")).purple().dimmed()
            );
        }
        println!();
    }
    
    output::info("To initialize a module:");
    println!("  {}     Initialize from registry", "declarch init <category>/<name>".green());
    println!("  {}          Create local module", "declarch init --local <name>".green());
    println!("  {}             Auto-confirm import", "declarch init <module> -y".green());
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
