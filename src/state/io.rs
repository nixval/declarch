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

/// Migrate state to fix duplicate keys and format issues
/// Returns true if migration was performed
fn migrate_state(state: &mut crate::state::types::State) -> Result<bool> {
    use crate::core::resolver;
    use std::collections::HashMap;

    let mut migrated = false;
    let mut new_packages: HashMap<String, crate::state::types::PackageState> = HashMap::new();

    // Track package signatures we've seen to detect duplicates
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (_key, pkg_state) in &state.packages {
        // Build the canonical key using current format
        let canonical_id = crate::core::types::PackageId {
            name: pkg_state.config_name.clone(),
            backend: pkg_state.backend.clone(),
        };
        let canonical_key = resolver::make_state_key(&canonical_id);

        // Check if we've already seen this package
        let signature = format!("{}:{}", pkg_state.backend, pkg_state.config_name);
        if seen.contains(&signature) {
            // Duplicate found - skip it
            migrated = true;
            continue;
        }

        seen.insert(signature);
        new_packages.insert(canonical_key, pkg_state.clone());
    }

    if migrated {
        state.packages = new_packages;
    }

    Ok(migrated)
}

pub fn load_state() -> Result<State> {
    let path = get_state_path()?;

    if !path.exists() {
        return Ok(State::default());
    }

    // Try to load the main state file
    let content = fs::read_to_string(&path);
    let state = match content {
        Ok(content) => {
            match serde_json::from_str::<State>(&content) {
                Ok(mut state) => {
                    // Migrate state to fix duplicate keys from old format
                    if migrate_state(&mut state)? {
                        // Save migrated state
                        let _ = save_state(&state);
                    }
                    return Ok(state);
                },
                Err(_) => {
                    // Main state file is corrupted, try to restore from backup
                    restore_from_backup(&path)?
                }
            }
        },
        Err(_) => {
            // Failed to read state file, try to restore from backup
            restore_from_backup(&path)?
        }
    };

    Ok(state)
}

/// Attempt to restore state from the most recent backup
fn restore_from_backup(state_path: &PathBuf) -> Result<State> {
    let dir = state_path.parent()
        .ok_or_else(|| DeclarchError::Other(
            format!("Invalid state path (no parent directory): {}", state_path.display())
        ))?;

    // Try backups in reverse order (most recent first)
    for i in 1..=3 {
        let backup_path = dir.join(format!("state.json.bak.{}", i));
        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path)
                .map_err(|e| DeclarchError::IoError {
                    path: backup_path.clone(),
                    source: e
                })?;

            match serde_json::from_str::<State>(&content) {
                Ok(state) => {
                    // Successfully restored from backup, restore the main file
                    let _ = fs::copy(&backup_path, state_path);
                    return Ok(state);
                },
                Err(_) => continue,
            }
        }
    }

    // All backups failed or don't exist, return default state
    Ok(State::default())
}

pub fn save_state(state: &State) -> Result<()> {
    let path = get_state_path()?;

    // Get parent directory - state paths should always have a parent
    let dir = path.parent()
        .ok_or_else(|| DeclarchError::Other(
            format!("Invalid state path (no parent directory): {}", path.display())
        ))?;

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
