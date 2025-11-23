use crate::utils::paths; 
use crate::config::loader;
use crate::ui as output; 
use crate::error::Result;
use colored::Colorize;

pub fn run(verbose: bool, check_duplicates: bool) -> Result<()> {
    output::header("Configuration Check");

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::error::DeclarchError::ConfigNotFound { path: config_path });
    }

    output::info(&format!("Entry point: {}", config_path.display()));

    let config = loader::load_root_config(&config_path)?;
    output::success("Syntax & Imports: OK");
    
    output::keyval("Unique Packages", &config.packages.len().to_string());
    output::keyval("Excludes", &config.excludes.len().to_string());  
    if verbose {
        output::separator();
        println!("{}", "Resolved Packages:".bold());
        
        let mut sorted_pkgs: Vec<_> = config.packages.keys().collect();
        sorted_pkgs.sort_by_key(|p| &p.name);

        for pkg in sorted_pkgs {
            output::indent(&pkg.to_string(), 2);
        }
        
        if !config.excludes.is_empty() {
            println!("\n{}", "Active Excludes:".bold());
            for ex in &config.excludes {
                output::indent(ex, 2);
            }
        }
    }
if check_duplicates {
        output::separator();
        output::info("Checking for duplicates...");
        
        let duplicates = config.get_duplicates();
        
        if duplicates.is_empty() {
            output::success("No duplicate declarations found.");
        } else {
            output::warning(&format!("Found {} duplicate package declarations:", duplicates.len()));
            
            for (pkg, sources) in duplicates {
                println!("  📦 {}", pkg.to_string().yellow().bold());
                for src in sources {
                    let display_path = src.file_name().unwrap_or_default().to_string_lossy();
                    println!("     └─ {}", display_path.dimmed());
                }
            }
            println!("\n{}", "Note: Duplicates are automatically deduplicated during sync.".italic());
        }
    }

    Ok(())
}
