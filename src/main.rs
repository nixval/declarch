mod config;
mod init;
mod module;
mod state;
mod system;

use clap::Parser;
use colored::*;
use config::{Cli, Commands, SyncArgs};
use miette::{miette, Result};
use std::collections::HashSet;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_dir = match cli.config {
        Some(path) => path,
        None => {
            dirs::config_dir()
                .ok_or_else(|| miette!("Failed to find config directory"))?
                .join("declarch")
        }
    };
    
    if !cli.quiet { println!("{}", format!("-> Using config directory: {:?}", config_dir).dimmed()); }
    
    // Check if config dir exists
    if !config_dir.exists() {
        return Err(miette!("Configuration directory not found at: {:?}. \nPlease create it.", config_dir));
    }

    match cli.command {
        Commands::Sync(args) => {
            if !config_dir.exists() {
                return Err(miette!("Configuration directory not found at: {:?}. \nPlease run 'declarch init' first.", config_dir));
            }
            run_sync(args, &config_dir, cli.quiet)?;
        }
        Commands::Module(args) => {
            if !config_dir.exists() {
                return Err(miette!("Configuration directory not found at: {:?}. \nPlease run 'declarch init' first.", config_dir));
            }
            module::run_module_command(args, &config_dir, cli.quiet)?;
        }
        Commands::Init => { // <-- 3. Tambahkan handler 'Init'
            init::run_init(&config_dir, cli.quiet)?;
        }
    }

    Ok(())
}

fn run_sync(args: SyncArgs, config_dir: &PathBuf, quiet: bool) -> miette::Result<()> {
    let (host_str, config_doc) = config::load_config(config_dir, quiet)?;
    let desired_reqs = config::load_modules(config_dir, &host_str, &config_doc, quiet)?;
    let installed_map = system::get_installed_packages(quiet)?;
    let managed_set = state::load_state(config_dir)?;
    
    if !quiet { println!("-> Loaded {} managed packages from state file.", managed_set.len()); }
    
    let mut to_install_names: Vec<String> = vec![];
    let mut desired_names: HashSet<String> = HashSet::new();

    if !quiet { println!("\n{}", "-> Calculating package diff...".blue()); }
    
    for req in &desired_reqs {
        desired_names.insert(req.name.clone());
        
        match installed_map.get(&req.name) {
            None => {
                if req.version.is_none() {
                    to_install_names.push(req.name.clone());
                } else {
                    println!("   - {} Pinned package '{}' not installed. Auto-downgrade not supported.", "[Warning]".yellow(), req.name.cyan());
                    println!("     -> Please install '{}' manually.", req.to_paru_string().cyan());
                }
            }
            Some(installed_ver) => {
                if let Some(req_ver) = &req.version {
                    if req_ver != installed_ver {
                        println!("   - {} Version mismatch for '{}'", "[Warning]".yellow(), req.name.cyan());
                        println!("     -> Wanted: {}, Have: {}", req_ver.green(), installed_ver.red());
                        println!("     -> Auto-downgrade not supported. Please fix manually.");
                    }
                }
            }
        }
    }

    let to_remove: Vec<String> = managed_set
        .difference(&desired_names)
        .cloned()
        .filter(|pkg_name| installed_map.contains_key(pkg_name))
        .collect();

    println!("\n{}", "-> Sync status:".blue());
    println!("   - {} new packages to install", to_install_names.len().to_string().green());
    println!("   - {} packages to remove (Prune)", to_remove.len().to_string().red());
    
    if args.prune {
        system::remove_packages(&to_remove, quiet)?;
        system::install_packages(&to_install_names, quiet)?;
    } else {
        system::install_packages(&to_install_names, quiet)?;
        if !quiet { println!("\n{}", "-> Pruning skipped. Use --prune to remove packages.".yellow()); }
    }
    
    if !quiet { println!("\n{}", "-> Updating state...".blue()); }
    
    state::save_state(config_dir, &desired_names)?;
    if !quiet { println!("-> State saved. {} packages now managed.", desired_names.len()); }

    Ok(())
}
