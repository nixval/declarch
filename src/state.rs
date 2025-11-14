// src/state.rs
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

type ManagedPackages = HashSet<String>;

#[derive(Serialize, Deserialize, Debug, Default)]
struct State {
    managed_packages: ManagedPackages,
}

fn get_state_path(config_dir: &Path) -> std::path::PathBuf {
    config_dir.join("state.json")
}

pub fn load_state(config_dir: &Path) -> Result<ManagedPackages> {
    let state_path = get_state_path(config_dir);
    if !state_path.exists() {
        return Ok(ManagedPackages::new());
    }

    let content = fs::read_to_string(&state_path).into_diagnostic()?;
    
    // This is the fix:
    if content.is_empty() {
        return Ok(ManagedPackages::new());
    }
    
    let state: State = serde_json::from_str(&content)
        .into_diagnostic()
        .map_err(|e| miette!("Failed to parse {:?}: {}", state_path, e))?;
    
    Ok(state.managed_packages)
}

pub fn save_state(config_dir: &Path, managed_packages: &HashSet<String>) -> Result<()> {
// ... (sisa file ini tidak berubah) ...
    let state = State {
        managed_packages: managed_packages.clone(),
    };
    
    let content = serde_json::to_string_pretty(&state).into_diagnostic()?;
    let state_path = get_state_path(config_dir);
    fs::write(state_path, content).into_diagnostic()?;
    
    Ok(())
}
