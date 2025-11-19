use std::path::PathBuf;
use std::fs;
use std::io::Write;
use crate::state::types::State;
use crate::utils::errors::{DeclarchError, Result};
use directories::ProjectDirs;

pub fn get_state_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "declarch", "declarch")
        .ok_or(DeclarchError::Other("Could not determine home directory".into()))?;
    
    // FIX: state_dir() returns Option, we must unwrap it
    let state_dir = proj_dirs.state_dir()
        .ok_or(DeclarchError::Other("System does not support state directory".into()))?;
    
    if !state_dir.exists() {
        fs::create_dir_all(state_dir)
            .map_err(|e| DeclarchError::FileWriteError { 
                path: state_dir.to_path_buf(), 
                reason: e.to_string() 
            })?;
    }

    Ok(state_dir.join("state.json"))
}

pub fn load_state() -> Result<State> {
    let path = get_state_path()?;
    
    if !path.exists() {
        return Ok(State::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| DeclarchError::FileReadError { 
            path: path.clone(), 
            reason: e.to_string() 
        })?;

    let state: State = serde_json::from_str(&content)
        .map_err(|e| DeclarchError::StateDeserializeError { reason: e.to_string() })?;

    Ok(state)
}

pub fn save_state(state: &State) -> Result<()> {
    let path = get_state_path()?;
    let dir = path.parent().unwrap();
    
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| DeclarchError::Other(format!("Failed to serialize state: {}", e)))?;

    let tmp_path = dir.join("state.tmp");
    let mut tmp_file = fs::File::create(&tmp_path)
        .map_err(|e| DeclarchError::FileWriteError { path: tmp_path.clone(), reason: e.to_string() })?;
    
    tmp_file.write_all(content.as_bytes())?;
    tmp_file.sync_all()?;

    if path.exists() {
        let backup_path = dir.join("state.json.bak");
        let _ = fs::copy(&path, &backup_path);
    }

    fs::rename(&tmp_path, &path)
        .map_err(|e| DeclarchError::FileWriteError { 
            path: path.clone(), 
            reason: format!("Atomic rename failed: {}", e) 
        })?;

    Ok(())
}

pub fn init_state(hostname: String) -> Result<State> {
    let mut state = State::default();
    state.meta.hostname = hostname;
    save_state(&state)?;
    Ok(state)
}
