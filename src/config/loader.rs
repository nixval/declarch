use super::types::PackageRequest;
use kdl::{KdlDocument, KdlValue};
use miette::{miette, IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path; 
fn as_string(val: &KdlValue) -> Result<String> {
// ... sisa file ini sudah benar ...
    val.as_string()
        .map(|s| s.to_string())
        .ok_or_else(|| miette!("Value was not a string"))
}

pub fn load_config(config_dir: &Path, quiet: bool) -> Result<(String, KdlDocument)> {
    let config_path = config_dir.join("config.kdl");
    
    if !quiet { println!("-> Reading config from: {:?}", config_path); }
    let content = fs::read_to_string(&config_path)
        .into_diagnostic()
        .map_err(|e| miette!("Failed to read config file at {:?}: {}", config_path, e))?;
    let doc: KdlDocument = content.parse().into_diagnostic()?;

    let host_node = doc.get("host")
        .ok_or_else(|| miette!("'host' node not found in {:?}", config_path))?;
    let host_entry = host_node.entries().get(0)
        .ok_or_else(|| miette!("'host' node needs a value"))?;
    let host_val = host_entry.value();
    let host_str = as_string(host_val)?;
    if !quiet { println!("-> Config loaded successfully for host: {}", host_str); }

    Ok((host_str, doc))
}

pub fn load_modules(
    config_dir: &Path,
    host_str: &str, 
    doc: &KdlDocument, 
    quiet: bool
) -> Result<Vec<PackageRequest>> {
    let mut all_packages: Vec<PackageRequest> = Vec::new();
    let mut excluded_packages: HashSet<String> = HashSet::new();
    
    let host_file_path = config_dir.join("hosts").join(format!("{}.kdl", host_str));

    if host_file_path.exists() {
        if !quiet { println!("-> Reading host config: {:?}", host_file_path); }
        let host_content = fs::read_to_string(&host_file_path).into_diagnostic()?;
        let host_doc: KdlDocument = host_content.parse().into_diagnostic()?;

        if let Some(packages_node) = host_doc.get("packages") {
            for pkg_entry in packages_node.entries() {
                let pkg_str = as_string(pkg_entry.value())?;
                all_packages.push(PackageRequest::from_kdl_string(&pkg_str));
            }
        }
        
        if let Some(exclude_node) = host_doc.get("exclude") {
            for pkg_entry in exclude_node.entries() {
                excluded_packages.insert(as_string(pkg_entry.value())?);
            }
        }

    } else {
        if !quiet { println!("-> No host-specific config found at {:?}. Skipping.", host_file_path); }
    }
    
    let modules_node = doc.get("enabled_modules")
        .ok_or_else(|| miette!("'enabled_modules' node not found in config.kdl"))?;
    
    if !quiet { println!("-> Reading modules..."); }
    
    let modules_dir = config_dir.join("modules");
    let mut enabled_modules_set: HashSet<String> = HashSet::new();
    let mut conflict_map: HashMap<String, HashSet<String>> = HashMap::new();

    for module_entry in modules_node.entries() {
        let module_name = as_string(module_entry.value())?;
        enabled_modules_set.insert(module_name.clone());

        let module_path = modules_dir.join(format!("{}.kdl", module_name));
        let module_content = fs::read_to_string(&module_path)
            .into_diagnostic()
            .map_err(|e| miette!("Failed to read module {:?}: {}", module_path, e))?;
        let module_doc: KdlDocument = module_content.parse().into_diagnostic()?;

        if let Some(desc_node) = module_doc.get("description") {
             if let Some(desc_entry) = desc_node.entries().get(0) {
                let desc_val = desc_entry.value();
                if !quiet { println!("   -> Processing module '{}': {}", module_name, as_string(desc_val)?); }
             }
        }

        if let Some(packages_node) = module_doc.get("packages") {
            for pkg_entry in packages_node.entries() {
                let pkg_str = as_string(pkg_entry.value())?;
                let pkg_req = PackageRequest::from_kdl_string(&pkg_str);
                
                if !excluded_packages.contains(&pkg_req.name) {
                    all_packages.push(pkg_req);
                }
            }
        }
        
        if let Some(conflicts_node) = module_doc.get("conflicts") {
            let conflicts: HashSet<String> = conflicts_node.entries().iter()
                .map(|entry| as_string(entry.value()))
                .collect::<Result<_>>()?;
            conflict_map.insert(module_name, conflicts);
        }
    }

    if !excluded_packages.is_empty() && !quiet {
        println!("-> Applied {} exclusion rules.", excluded_packages.len());
    }
    
    for module_name in &enabled_modules_set {
        if let Some(conflicts) = conflict_map.get(module_name) {
            for conflicting_module in conflicts {
                if enabled_modules_set.contains(conflicting_module) {
                    return Err(miette!(
                        "Conflict detected: Module '{}' conflicts with '{}'. Please disable one of them.",
                        module_name,
                        conflicting_module
                    ));
                }
            }
        }
    }

    if !quiet {
        println!("\n-> All modules processed successfully.");
        println!("-> Total packages collected: {}", all_packages.len());
    }
    
    Ok(all_packages)
}
