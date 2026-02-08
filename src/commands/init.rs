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
    pub backend: Option<String>,
    pub force: bool,
    pub skip_soar_install: bool, // Deprecated: kept for compatibility
}

pub fn run(options: InitOptions) -> Result<()> {
    // CASE A: MODULE INITIALIZATION
    if let Some(target_path) = options.path {
        return init_module(&target_path, options.force);
    }

    // CASE A2: BACKEND INITIALIZATION
    if let Some(backend_name) = options.backend {
        return init_backend(&backend_name, options.force);
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
    output::header("Initializing Backend");

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

/// Display backend meta information
fn display_backend_meta(_content: &str, name: &str) {
    // Get description based on known backends
    let description = match name {
        "cargo" => "Rust package manager",
        "npm" => "Node.js package manager",
        "pip" | "pip3" => "Python package installer",
        "pnpm" => "Fast, disk space efficient package manager",
        "yarn" => "Fast, reliable dependency management",
        "gem" => "Ruby package manager",
        "go" | "golang" => "Go package manager",
        "composer" => "PHP package manager",
        "luarocks" => "Lua package manager",
        "cabal" => "Haskell package manager",
        "stack" => "Haskell build tool",
        "conan" => "C/C++ package manager",
        "vcpkg" => "C++ package manager",
        "nix" => "Nix package manager",
        "guix" => "GNU Guix package manager",
        "snap" => "Universal Linux package",
        "appimage" => "Portable Linux apps",
        _ => "Custom package manager backend",
    };

    output::separator();
    println!("{}", "Backend Information:".bold().cyan());
    println!("  {}", name.bold());
    println!();
    println!("  {}", description.dimmed());
    println!();
    println!("  {}", format!("Type: Package Manager").green());
    output::separator();
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
fn generate_backend_template(name: &str) -> String {
    // Try to guess the package manager based on name
    let (binary, pm_name, description, homepage): (&str, &str, String, String) = match name {
        "cargo" => ("cargo", "Cargo", "Rust package manager".to_string(), "https://doc.rust-lang.org/cargo/".to_string()),
        "npm" => ("npm", "NPM", "Node.js package manager".to_string(), "https://www.npmjs.com".to_string()),
        "pip" | "pip3" => ("pip3", "pip", "Python package installer".to_string(), "https://pip.pypa.io".to_string()),
        "pnpm" => ("pnpm", "pnpm", "Fast, disk space efficient package manager".to_string(), "https://pnpm.io".to_string()),
        "yarn" => ("yarn", "Yarn", "Fast, reliable, and secure dependency management".to_string(), "https://yarnpkg.com".to_string()),
        "gem" => ("gem", "RubyGems", "Ruby package manager".to_string(), "https://rubygems.org".to_string()),
        "go" | "golang" => ("go", "Go Modules", "Go package manager".to_string(), "https://go.dev".to_string()),
        "composer" => ("composer", "Composer", "PHP package manager".to_string(), "https://getcomposer.org".to_string()),
        "luarocks" => ("luarocks", "LuaRocks", "Lua package manager".to_string(), "https://luarocks.org".to_string()),
        "cabal" => ("cabal", "Cabal", "Haskell package manager".to_string(), "https://www.haskell.org/cabal/".to_string()),
        "stack" => ("stack", "Stack", "Haskell build tool".to_string(), "https://docs.haskellstack.org".to_string()),
        "conan" => ("conan", "Conan", "C/C++ package manager".to_string(), "https://conan.io".to_string()),
        "vcpkg" => ("vcpkg", "vcpkg", "C++ package manager".to_string(), "https://vcpkg.io".to_string()),
        "nix" => ("nix-env", "Nix", "Nix package manager".to_string(), "https://nixos.org".to_string()),
        "guix" => ("guix", "Guix", "GNU Guix package manager".to_string(), "https://guix.gnu.org".to_string()),
        "snap" => ("snap", "Snap", "Universal Linux package".to_string(), "https://snapcraft.io".to_string()),
        "appimage" => ("appimage", "AppImage", "Portable Linux apps".to_string(), "https://appimage.org".to_string()),
        _ => (name, name, format!("{} package manager", name), String::new()),
    };

    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let homepage_line = if homepage.is_empty() {
        String::new()
    } else {
        format!("        homepage \"{}\"", &homepage)
    };

    format!(
        r#"// {name} - {description}
// 
// This is a template backend configuration for declarch.
// Customize the commands below to match your package manager's syntax.

backend "{name}" {{
    meta {{
        title "{pm_name}"
        description "{description}"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "{name}"
{homepage_line}
        license "Unknown"
        created "{date}"
        platforms "linux"
        requires "{binary}"
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
    
    // Fallback backend if binary not found (optional, v0.6+)
    // fallback "apt"
}}
"#,
        name = name,
        pm_name = pm_name,
        description = description,
        binary = binary,
        date = current_date,
        homepage_line = homepage_line
    )
}

fn init_module(target_path: &str, force: bool) -> Result<()> {
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

    // STRATEGY A: Hardcoded Template (Fastest, Offline)
    let content = if let Some(local_tmpl) = utils::templates::get_template_by_name(&slug) {
        local_tmpl
    }
    // STRATEGY B: Remote Registry (The "Marketplace")
    else {
        // Use the full target_path (e.g., "hyprland/niri-nico") for remote fetch
        match remote::fetch_module_content(target_path) {
            Ok(remote_content) => remote_content,
            Err(e) => {
                output::warning(&format!("Remote fetch failed: {}", e));
                utils::templates::default_module(&slug)
            }
        }
    };

    // Display module meta information before proceeding
    display_module_meta(&content);

    // 4. Write File
    fs::write(&full_path, &content)?;

    // 5. AUTO INJECT IMPORT
    let root_config_path = paths::config_file()?;
    let import_path = modules_path.to_string_lossy().replace("\\", "/");
    inject_import_to_root(&root_config_path, &import_path, force)?;

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
fn inject_import_to_root(config_path: &Path, import_path: &str, force: bool) -> Result<()> {
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

    // Prompt for consent unless force is active
    if !force
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
