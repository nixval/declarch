use crate::utils::paths;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use std::fs;
use colored::Colorize;

#[derive(Debug)]
pub struct InitOptions {
    pub host: Option<String>,
    pub force: bool,
}

pub fn run(options: InitOptions) -> Result<()> {
    output::header("Initializing declarch");

    let config_dir = paths::config_dir()?;
    let config_file = paths::config_file()?;
    
    if config_file.exists() && !options.force {
        output::warning("Configuration already exists.");
        output::info(&format!("Location: {}", config_file.display()));
        return Ok(());
    }

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        output::success(&format!("Created config directory: {}", config_dir.display()));
    }

    let hostname = options.host.unwrap_or_else(|| {
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let template = format!(
        r#"// declarch configuration
// Created for host: {}

// Import other module files (optional)
// imports {{
//     "modules/common.kdl"
// }}

packages {{
    // Add packages here
    git
    vim
    // flatpak:obsidian
}}

excludes {{
    // Add packages to exclude
}}
"#, 
        hostname
    );

    fs::write(&config_file, template)?;
    output::success(&format!("Created config file: {}", config_file.display()));

    let _state = state::io::init_state(hostname.clone())?;
    output::success(&format!("Initialized state for host: {}", hostname.green()));

    Ok(())
}
