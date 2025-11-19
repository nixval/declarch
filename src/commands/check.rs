use crate::utils::{output, errors::Result, paths};
use crate::config::loader;
use colored::Colorize;

pub fn run(verbose: bool) -> Result<()> {
    output::header("Configuration Check");

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::utils::errors::DeclarchError::ConfigNotFound { path: config_path });
    }
    
    output::info(&format!("Entry point: {}", config_path.display()));

    match loader::load_root_config(&config_path) {
        Ok(config) => {
            output::success("Syntax & Imports: OK");
            
            output::separator();
            println!("Summary:");
            output::keyval("Packages", &config.packages.len().to_string());
            output::keyval("Excludes", &config.excludes.len().to_string());

            if verbose {
                output::separator();
                println!("{}", "Resolved Packages:".bold());
                for pkg in config.packages {
                    output::indent(&pkg, 2);
                }
                
                if !config.excludes.is_empty() {
                    println!("\n{}", "Active Excludes:".bold());
                    for ex in config.excludes {
                        output::indent(&ex, 2);
                    }
                }
            }
        }
        Err(e) => {
            output::error("Configuration invalid!");
            return Err(e);
        }
    }

    Ok(())
}
