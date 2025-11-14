use crate::config::{ModuleArgs, ModuleCommand};
use colored::*;
use kdl::{KdlDocument, KdlEntry, KdlValue};
use miette::{miette, IntoDiagnostic, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn run_module_command(args: ModuleArgs, config_dir: &Path, quiet: bool) -> Result<()> {
    match args.command {
        ModuleCommand::List => list_modules(config_dir, quiet)?,
        ModuleCommand::Enable { name } => enable_module(config_dir, &name, quiet)?,
        ModuleCommand::Disable { name } => disable_module(config_dir, &name, quiet)?,
    }
    Ok(())
}

fn list_modules(config_dir: &Path, quiet: bool) -> Result<()> {
    if !quiet { println!("{}", "-> Loading configuration...".blue()); }
    let config_path = config_dir.join("config.kdl");
    let config_content = fs::read_to_string(config_path).into_diagnostic()?;
    let config_doc: KdlDocument = config_content.parse().into_diagnostic()?;
    
    let enabled_node = config_doc.get("enabled_modules")
        .ok_or_else(|| miette!("'enabled_modules' node not found in config.kdl"))?;
    
    let enabled_set: HashSet<String> = enabled_node.entries().iter()
        .filter_map(|entry| entry.value().as_string().map(String::from))
        .collect();

    if !quiet { println!("{}", "-> Scanning for available modules...".blue()); }
    let modules_dir = config_dir.join("modules");
    let mut modules_found = false;
    
    if let Ok(entries) = fs::read_dir(modules_dir) {
        for entry in entries {
            let entry = entry.into_diagnostic()?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |s| s == "kdl") {
                modules_found = true;
                let module_name = path.file_stem()
                    .ok_or_else(|| miette!("Failed to get file stem for {:?}", path))?
                    .to_string_lossy().to_string();
                
                if enabled_set.contains(&module_name) {
                    println!(" - {} (enabled)", module_name.green());
                } else {
                    println!(" - {} (disabled)", module_name.dimmed());
                }
            }
        }
    }

    if !modules_found {
        println!("{}", "No modules found in packages/modules/".yellow());
    }
    
    Ok(())
}

fn enable_module(config_dir: &Path, name: &str, quiet: bool) -> Result<()> {
    if !quiet { println!("-> Enabling module '{}'...", name.cyan()); }
    
    let module_path = config_dir.join("modules").join(format!("{}.kdl", name));
    if !fs::metadata(&module_path).is_ok() {
        return Err(miette!("Module file not found: {:?}. Aborting.", module_path));
    }
    
    let config_path = config_dir.join("config.kdl");
    let config_content = fs::read_to_string(&config_path).into_diagnostic()?;
    let mut config_doc: KdlDocument = config_content.parse().into_diagnostic()?;
    
    let enabled_node = config_doc.get_mut("enabled_modules")
        .ok_or_else(|| miette!("'enabled_modules' node not found in config.kdl"))?;
    
    let already_enabled = enabled_node.entries().iter()
        .any(|entry| entry.value().as_string() == Some(name));
    
    if already_enabled {
        println!("{}", "Module is already enabled.".yellow());
        return Ok(());
    }
    
    enabled_node.push(KdlEntry::new(KdlValue::from(name.to_string())));
    
    fs::write(config_path, config_doc.to_string()).into_diagnostic()?;
    
    if !quiet { println!("{}", "Module enabled successfully. Run 'declarch sync' to apply.".green()); }
    Ok(())
}

fn disable_module(config_dir: &Path, name: &str, quiet: bool) -> Result<()> {
    if !quiet { println!("-> Disabling module '{}'...", name.cyan()); }
    
    let config_path = config_dir.join("config.kdl");
    let config_content = fs::read_to_string(&config_path).into_diagnostic()?;
    let mut config_doc: KdlDocument = config_content.parse().into_diagnostic()?;
    
    let enabled_node = config_doc.get_mut("enabled_modules")
        .ok_or_else(|| miette!("'enabled_modules' node not found in config.kdl"))?;
    
    let entry_index = enabled_node.entries().iter()
        .position(|entry| entry.value().as_string() == Some(name));
    
    if let Some(index) = entry_index {
        enabled_node.remove(index);
        fs::write(config_path, config_doc.to_string()).into_diagnostic()?;
        if !quiet { println!("{}", "Module disabled successfully. Run 'declarch sync --prune' to apply.".green()); }
    } else {
        println!("{}", "Module is not enabled.".yellow());
    }
    
    Ok(())
}
