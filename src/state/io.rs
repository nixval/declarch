use std::path::PathBuf;
use std::fs;
use std::io::Write;
use crate::state::types::State;
use crate::error::{DeclarchError, Result};
use directories::ProjectDirs;

pub fn get_state_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "declarch", "declarch")
        .ok_or(DeclarchError::Other("Could not determine home directory".into()))?;
    
    let state_dir = proj_dirs.state_dir()
        .ok_or(DeclarchError::Other("System does not support state directory".into()))?;
    
    if !state_dir.exists() {
        fs::create_dir_all(state_dir)
            .map_err(|e| DeclarchError::IoError { 
                path: state_dir.to_path_buf(), 
                source: e 
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
        .map_err(|e| DeclarchError::IoError { 
            path: path.clone(), 
            source: e 
        })?;

    let state: State = serde_json::from_str(&content)?;

    Ok(state)
}

pub fn save_state(state: &State) -> Result<()> {
    let path = get_state_path()?;
    let dir = path.parent().unwrap();
    
    // --- ROTATING BACKUP LOGIC (Keep last 3 versions) ---
    // Shift: .bak.2 -> .bak.3
    // Shift: .bak.1 -> .bak.2
    // Copy:  current -> .bak.1
    
    if path.exists() {
        let max_backups = 3;
        for i in (1..max_backups).rev() {
            let old_bak = dir.join(format!("state.json.bak.{}", i));
            let new_bak = dir.join(format!("state.json.bak.{}", i + 1));
            if old_bak.exists() {
                let _ = fs::rename(&old_bak, &new_bak);
            }
        }

        let first_bak = dir.join("state.json.bak.1");
        let _ = fs::copy(&path, &first_bak);
    }
    // ----------------------------------------------------

    let content = serde_json::to_string_pretty(state)?;

    let tmp_path = dir.join("state.tmp");
    let mut tmp_file = fs::File::create(&tmp_path)
        .map_err(|e| DeclarchError::IoError { path: tmp_path.clone(), source: e })?;
    
    tmp_file.write_all(content.as_bytes())?;
    tmp_file.sync_all()?;

    fs::rename(&tmp_path, &path)
        .map_err(|e| DeclarchError::IoError { 
            path: path.clone(), 
            source: e
        })?;

    Ok(())
}

pub fn init_state(hostname: String) -> Result<State> {
    let mut state = State::default();
    state.meta.hostname = hostname;
    save_state(&state)?;
    Ok(state)
}
