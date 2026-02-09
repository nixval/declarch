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

/// Default backends.kdl content
const DEFAULT_BACKENDS_KDL: &str = r#"// Backend Aggregator
// 
// This file imports all custom backend configurations.
// Add new backends with: declarch init --backend <name>

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
    output::header("Initializing declarch root");

    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    if config_file.exists() && !force {
        output::warning("Configuration already exists.");
        output::info(&format!("Location: {}", config_file.display()));
        return Ok(());
    }

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        output::success(&format!(
            "Created config directory: {}",
            config_dir.display()
        ));
    }

    let hostname = host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    // Create backends.kdl first (so it's referenced in template)
    let backends_kdl_path = config_dir.join("backends.kdl");
    if !backends_kdl_path.exists() {
        fs::write(&backends_kdl_path, DEFAULT_BACKENDS_KDL)?;
        output::success(&format!(
            "Created backends aggregator: {}",
            backends_kdl_path.display()
        ));
    }

    let template = utils::templates::default_host(&hostname);

    fs::write(&config_file, template)?;
    output::success(&format!("Created config file: {}", config_file.display()));

    // Create modules/base.kdl with default template
    let modules_dir = config_dir.join("modules");
    if !modules_dir.exists() {
        fs::create_dir_all(&modules_dir)?;
        output::success(&format!(
            "Created modules directory: {}",
            modules_dir.display()
        ));
    }

    let base_module_path = modules_dir.join(format!("base.{}", CONFIG_EXTENSION));
    if !base_module_path.exists() {
        let base_template = utils::templates::get_template_by_name("base")
            .unwrap_or_else(|| utils::templates::default_module("base"));
        fs::write(&base_module_path, base_template)?;
        output::success(&format!(
            "Created base module: {}",
            base_module_path.display()
        ));
    }

    let _state = state::io::init_state(hostname.clone())?;
    output::success(&format!("Initialized state for host: {}", hostname.green()));

    Ok(())
}

/// Initialize a new backend configuration file
fn init_backend(backend_name: &str, force: bool) -> Result<()> {

    let root_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;

    // STEP 1: Auto-initialize root if not exists
    if !config_file.exists() {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        // Create root directory
        fs::create_dir_all(&root_dir)?;

        // Create default config
        let template = utils::templates::default_host(&hostname);
        fs::write(&config_file, template)?;
        output::success(&format!("Created config file: {}", config_file.display()));

        // Initialize state
        let _state = state::io::init_state(hostname)?;
    }

    // STEP 2: Create backends directory
    let backends_dir = root_dir.join("backends");
    if !backends_dir.exists() {
        fs::create_dir_all(&backends_dir)?;
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

    // STEP 3: Create backend file
    let backend_file = backends_dir.join(format!("{}.kdl", sanitized_name));

    if backend_file.exists() && !force {
        output::warning(&format!(
            "Backend '{}' already exists: {}",
            sanitized_name,
            backend_file.display()
        ));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    // Try to fetch from registry first, then fallback to template
    let template = match remote::fetch_backend_content(&sanitized_name) {
        Ok(content) => {
            output::success(&format!(
                "Fetched backend '{}' from registry",
                sanitized_name
            ));
            content
        }
        Err(_) => {
            output::info(&format!(
                "Backend '{}' not found in registry, using local template",
                sanitized_name
            ));
            generate_backend_template(&sanitized_name)
        }
    };
    fs::write(&backend_file, &template)?;

    // STEP 4: Display backend info (like module info)
    display_backend_meta(&template, &sanitized_name);

    // STEP 5: Add to backends.kdl (aggregator pattern)
    let backends_kdl_path = root_dir.join("backends.kdl");
    let backend_entry = format!("backends/{}.kdl", sanitized_name);
    
    // Create backends.kdl if not exists
    if !backends_kdl_path.exists() {
        fs::write(&backends_kdl_path, DEFAULT_BACKENDS_KDL)?;
    }

    // Prompt for consent (like module init)
    if !force
        && !output::prompt_yes_no(&format!(
            "Add '{}' to backends.kdl?",
            backend_entry
        ))
    {
        output::info("Skipping auto-import. You can add it manually to backends.kdl.");
        output::success(&format!(
            "Created backend configuration: {}",
            backend_file.display()
        ));
        return Ok(());
    }

    // Add import to backends.kdl
    inject_import_to_backends_kdl(&backends_kdl_path, &backend_entry)?;

    output::success(&format!(
        "Created backend configuration: {}",
        backend_file.display()
    ));
    output::info("Edit this file to customize the backend behavior.");
    output::info("Run 'declarch check validate' to verify the backend configuration.");

    Ok(())
}

/// Display backend meta information from KDL content
fn display_backend_meta(content: &str, name: &str) {
    // Parse KDL to extract meta fields
    let meta_fields = extract_meta_fields(content);
    
    output::separator();
    println!("{}", "Backend:".bold().cyan());
    println!("  {}", name.bold());
    
    // Display meta fields if available
    if let Some(title) = meta_fields.get("title") {
        println!("  {}: {}", "Title".dimmed(), title);
    }
    if let Some(description) = meta_fields.get("description") {
        println!("  {}: {}", "Description".dimmed(), description);
    }
    if let Some(maintained) = meta_fields.get("maintained") {
        println!("  {}: {}", "Maintained by".dimmed(), maintained);
    }
    if let Some(homepage) = meta_fields.get("homepage") {
        if homepage != "-" && !homepage.is_empty() {
            println!("  {}: {}", "Homepage".dimmed(), homepage.italic());
        }
    }
    if let Some(platforms) = meta_fields.get("platforms") {
        println!("  {}: {}", "Platforms".dimmed(), platforms);
    }
    
    output::separator();
}

/// Extract meta fields from KDL content
fn extract_meta_fields(content: &str) -> std::collections::HashMap<String, String> {
    let mut fields = std::collections::HashMap::new();
    
    // Simple regex-based extraction for meta fields
    // Matches: field_name "value" or field_name "value" "value2"
    let re = regex::Regex::new(r#"(?m)^\s+(\w+)\s+"([^"]+)""#).unwrap();
    
    // Find meta block
    let meta_start = content.find("meta {");
    let meta_end = content.find("}");
    
    if let (Some(start), Some(end)) = (meta_start, meta_end) {
        if start < end {
            let meta_block = &content[start..end];
            for cap in re.captures_iter(meta_block) {
                let key = cap[1].to_string();
                let value = cap[2].to_string();
                // Skip fields with "-" value
                if value != "-" {
                    fields.insert(key, value);
                }
            }
        }
    }
    
    fields
}

/// Inject import into backends.kdl
fn inject_import_to_backends_kdl(backends_kdl_path: &Path, import_path: &str) -> Result<()> {
    let content = fs::read_to_string(backends_kdl_path)?;

    // Check if already exists
    if content.contains(import_path) {
        output::info(&format!(
            "Backend '{}' is already referenced in backends.kdl.",
            import_path
        ));
        return Ok(());
    }

    // Add import line
    let import_line = format!("import \"{}\"", import_path);
    let new_content = format!("{}\n{}", content.trim_end(), import_line);

    fs::write(backends_kdl_path, new_content)?;
    output::success("Added backend import to backends.kdl");

    Ok(())
}

/// Generate a backend template based on the backend name
/// Unknown metadata values are set to "-" and will be skipped in output
fn generate_backend_template(name: &str) -> String {
    // Backend-agnostic template generation
    // Unknown values use "-" which means "skip this field"
    // Special cases: aur, pacman, flatpak have specific fallback chains

    // Determine binary and fallback chain for known backends
    let (binary, fallback) = match name {
        "aur" => ("paru", Some("yay")),  // paru -> yay -> pacman
        "yay" => ("yay", Some("pacman")),
        "paru" => ("paru", Some("yay")),
        "flatpak" => ("flatpak", None),
        _ => (name, None),
    };
    
    let pm_name = name;
    let description = format!("{} package manager", name);
    
    // Use "-" for unknown optional fields (will be skipped in output)
    let version = "-";
    let author = "-";
    let maintained = "nixval";
    let url = "-";
    let homepage = "-";
    let license = "-";
    let install_guide = "-";

    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Build meta section dynamically, skipping "-" values
    let mut meta_lines = vec![
        format!(r#"        title "{}""#, pm_name),
        format!(r#"        description "{}""#, description),
    ];
    
    if version != "-" {
        meta_lines.push(format!(r#"        version "{}""#, version));
    }
    
    if author != "-" {
        meta_lines.push(format!(r#"        author "{}""#, author));
    }
    
    meta_lines.push(format!(r#"        maintained "{}""#, maintained));
    meta_lines.push(format!(r#"        tags "package-manager" "{}""#, name));
    
    if url != "-" {
        meta_lines.push(format!(r#"        url "{}""#, url));
    }
    
    if homepage != "-" {
        meta_lines.push(format!(r#"        homepage "{}""#, homepage));
    }
    
    if license != "-" {
        meta_lines.push(format!(r#"        license "{}""#, license));
    }
    
    meta_lines.push(format!(r#"        created "{}""#, current_date));
    meta_lines.push(r#"        platforms "linux""#.to_string());
    meta_lines.push(format!(r#"        requires "{}""#, binary));
    
    if install_guide != "-" {
        meta_lines.push(format!(r#"        install-guide "{}""#, install_guide));
    }

    let meta_section = meta_lines.join("\n");
    
    // Build fallback section if needed
    let fallback_section = fallback.map(|fb| {
        format!(r#"
    // Fallback backend if binary not found
    fallback "{}"
"#, fb)
    }).unwrap_or_default();

    // Special template for AUR helpers (paru/yay)
    if name == "aur" || name == "paru" || name == "yay" {
        return format!(
            r#"// {name} - AUR Helper
// 
// Supports paru/yay as primary, with pacman as ultimate fallback.

backend "{name}" {{
    meta {{
{meta_section}
    }}
    
    // Try paru first, then yay
    binary "paru" "yay"
    
    // List all installed packages (official repo + AUR)
    // Use -Q (all) instead of -Qm (AUR only) for broader package detection
    list "{{binary}} -Q" {{
        format whitespace
        name_col 0
    }}
    
    // Install from AUR or official repos
    install "{{binary}} -S --needed {{packages}}"
    
    // Remove packages
    remove "{{binary}} -R {{packages}}"
    
    // Search packages
    // search "{{binary}} -Ss {{query}}" {{
    //     format whitespace
    //     name_col 0
    //     desc_col 1
    // }}
    
    // Auto-confirmation (use with caution)
    // noconfirm "--noconfirm"
    
    // Fallback chain: aur -> pacman (if no AUR helper installed)
    fallback "pacman"
}}
"#,
            name = name,
            meta_section = meta_section
        );
    }

    format!(
        r#"// {name} - {description}
// 
// This is a template backend configuration for declarch.
// Customize the commands below to match your package manager's syntax.

backend "{name}" {{
    meta {{
{meta_section}
    }}
    
    // The binary to use (can specify multiple alternatives)
    binary "{binary}"
    
    // Command to list installed packages
    // Supported formats: tsv, whitespace, json, regex
    list "{binary} list" {{
        format whitespace
        name_col 0
        // version_col 1  // Uncomment if version is available
    }}
    
    // Install command - {{packages}} will be replaced with package names
    install "{binary} install {{packages}}"
    
    // Remove command
    remove "{binary} remove {{packages}}"
    
    // Search command (optional but recommended)
    // search "{binary} search {{query}}" {{
    //     format whitespace
    //     name_col 0
    //     desc_col 1
    // }}
    
    // Auto-confirmation flag (optional)
    // noconfirm "-y"
    
    // Whether this backend requires sudo (optional)
    // needs_sudo true
{fallback_section}
}}
"#,
        name = name,
        description = description,
        binary = binary,
        meta_section = meta_section,
        fallback_section = fallback_section
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
