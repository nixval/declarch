use crate::error::{DeclarchError, Result};
use crate::state::types::State;
use crate::ui;
use directories::ProjectDirs;
use fs2::FileExt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum age of a lock file in seconds before it's considered stale
const LOCK_TIMEOUT_SECONDS: u64 = 300; // 5 minutes

/// Lock file handle - keeps lock active until dropped
pub struct StateLock {
    _file: std::fs::File,
    path: PathBuf,
}

impl Drop for StateLock {
    fn drop(&mut self) {
        // Remove lock file when lock is released
        let _ = fs::remove_file(&self.path);
    }
}

/// Acquire exclusive lock for state operations
/// Returns lock handle that must be kept alive during the operation
/// Returns error if another process is already running
pub fn acquire_lock() -> Result<StateLock> {
    let path = get_state_path()?;
    let dir = path.parent().ok_or_else(|| {
        DeclarchError::Other("Could not determine state directory".into())
    })?;
    let lock_path = dir.join("state.lock");
    
    // Check if lock file exists
    if lock_path.exists() {
        // Check if lock file is stale
        let metadata = fs::metadata(&lock_path)?;
        
        if let Ok(modified) = metadata.modified() {
            let now = SystemTime::now();
            if let Ok(age) = now.duration_since(modified) {
                if age.as_secs() > LOCK_TIMEOUT_SECONDS {
                    ui::warning("Removing stale lock file (older than 5 minutes)");
                    let _ = fs::remove_file(&lock_path);
                } else {
                    // Lock is still valid - check if it's actually held
                    let existing_file = OpenOptions::new()
                        .write(true)
                        .open(&lock_path)?;
                    
                    match existing_file.try_lock_exclusive() {
                        Ok(()) => {
                            // We got the lock, previous process died without cleaning up
                            let _ = fs::remove_file(&lock_path);
                        }
                        Err(_) => {
                            return Err(DeclarchError::Other(format!(
                                "Another declarch process is currently running.\n\
                                 Lock file: {}\n\
                                 Wait for it to complete, or delete the lock file if you're sure no other process is running.",
                                lock_path.display()
                            )));
                        }
                    }
                }
            }
        }
    }
    
    // Create and lock the file
    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .map_err(|e| DeclarchError::IoError {
            path: lock_path.clone(),
            source: e,
        })?;
    
    lock_file.lock_exclusive().map_err(|e| DeclarchError::Other(format!(
        "Failed to lock state file: {}",
        e
    )))?;
    
    // Write PID to lock file for debugging
    let pid = std::process::id();
    let _ = writeln!(&lock_file, "{}", pid);
    
    Ok(StateLock {
        _file: lock_file,
        path: lock_path,
    })
}

/// Validate state integrity and report issues
pub fn validate_state_integrity(state: &State) -> Vec<String> {
    let mut issues = Vec::new();
    use std::collections::HashSet;
    
    // Check for duplicate package signatures
    let mut seen = HashSet::new();
    for pkg_state in state.packages.values() {
        let signature = format!("{}:{}", pkg_state.backend, pkg_state.config_name);
        if seen.contains(&signature) {
            issues.push(format!(
                "Duplicate package: {} in backend {}",
                pkg_state.config_name, pkg_state.backend
            ));
        }
        seen.insert(signature);
    }
    
    // Check for empty package names
    for (key, pkg_state) in &state.packages {
        if pkg_state.config_name.is_empty() {
            issues.push(format!("Empty package name in key: {}", key));
        }
    }
    
    // Check for future timestamps
    let now = SystemTime::now();
    if let Ok(last_sync) = pkg_state_timestamp(&state.meta.last_sync) {
        if last_sync > now {
            issues.push("Last sync timestamp is in the future".to_string());
        }
    }
    
    issues
}

fn pkg_state_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> Result<SystemTime> {
    let secs = dt.timestamp() as u64;
    Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

pub fn get_state_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "declarch", "declarch").ok_or(
        DeclarchError::PathError("Could not determine home directory".into()),
    )?;

    let state_dir = proj_dirs.state_dir().ok_or(DeclarchError::PathError(
        "System does not support state directory".into(),
    ))?;

    if !state_dir.exists() {
        fs::create_dir_all(state_dir).map_err(|e| DeclarchError::IoError {
            path: state_dir.to_path_buf(),
            source: e,
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

    for pkg_state in state.packages.values() {
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
                    // Validate integrity
                    let issues = validate_state_integrity(&state);
                    if !issues.is_empty() {
                        ui::warning("State integrity issues detected:");
                        for issue in &issues {
                            ui::indent(&format!("• {}", issue), 2);
                        }
                    }
                    
                    // Migrate state to fix duplicate keys from old format
                    if migrate_state(&mut state)? {
                        ui::info("State migrated to fix duplicate keys");
                        // Save migrated state
                        let _ = save_state(&state);
                    }
                    return Ok(state);
                }
                Err(e) => {
                    ui::error(&format!("State file corrupted: {}", e));
                    ui::info("Attempting to restore from backup...");
                    // Main state file is corrupted, try to restore from backup
                    match restore_from_backup(&path) {
                        Ok(state) => {
                            ui::success("State restored from backup successfully");
                            state
                        }
                        Err(restore_err) => {
                            ui::warning(&format!("Failed to restore from backup: {}", restore_err));
                            ui::info("Using default state");
                            State::default()
                        }
                    }
                }
            }
        }
        Err(e) => {
            ui::error(&format!("Failed to read state file: {}", e));
            ui::info("Attempting to restore from backup...");
            // Failed to read state file, try to restore from backup
            match restore_from_backup(&path) {
                Ok(state) => {
                    ui::success("State restored from backup successfully");
                    state
                }
                Err(restore_err) => {
                    ui::warning(&format!("Failed to restore from backup: {}", restore_err));
                    ui::info("Using default state");
                    State::default()
                }
            }
        }
    };

    Ok(state)
}

/// Attempt to restore state from the most recent backup
fn restore_from_backup(state_path: &PathBuf) -> Result<State> {
    let dir = state_path.parent().ok_or_else(|| {
        DeclarchError::PathError(format!(
            "Invalid state path (no parent directory): {}",
            state_path.display()
        ))
    })?;

    // Try backups in reverse order (most recent first)
    for i in 1..=3 {
        let backup_path = dir.join(format!("state.json.bak.{}", i));
        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path).map_err(|e| DeclarchError::IoError {
                path: backup_path.clone(),
                source: e,
            })?;

            match serde_json::from_str::<State>(&content) {
                Ok(state) => {
                    // Successfully restored from backup, restore the main file
                    let _ = fs::copy(&backup_path, state_path);
                    return Ok(state);
                }
                Err(_) => continue,
            }
        }
    }

    // All backups failed or don't exist, return default state
    Ok(State::default())
}

/// Rotate backup files, keeping last 3 versions
///
/// # Errors
/// Returns an error only if the initial backup copy fails. Rotation failures
/// are logged as warnings but don't prevent the operation from continuing.
fn rotate_backups(dir: &Path, path: &Path) -> Result<()> {
    // --- ROTATING BACKUP LOGIC (Keep last 3 versions) ---
    // Shift: .bak.2 -> .bak.3
    // Shift: .bak.1 -> .bak.2
    // Copy:  current -> .bak.1

    if path.exists() {
        let max_backups = 3;
        for i in (1..max_backups).rev() {
            let old_bak = dir.join(format!("state.json.bak.{}", i));
            let new_bak = dir.join(format!("state.json.bak.{}", i + 1));
            if old_bak.exists()
                && let Err(e) = fs::rename(&old_bak, &new_bak) {
                    ui::warning(&format!(
                        "Failed to rotate backup {} -> {}: {}",
                        old_bak.display(),
                        new_bak.display(),
                        e
                    ));
                }
        }

        let first_bak = dir.join("state.json.bak.1");
        if let Err(e) = fs::copy(path, &first_bak) {
            return Err(DeclarchError::IoError {
                path: first_bak,
                source: e,
            });
        }
    }
    // ----------------------------------------------------

    Ok(())
}

pub fn save_state(state: &State) -> Result<()> {
    let path = get_state_path()?;

    // Get parent directory - state paths should always have a parent
    let dir = path.parent().ok_or_else(|| {
        DeclarchError::PathError(format!(
            "Invalid state path (no parent directory): {}",
            path.display()
        ))
    })?;

    // Perform backup rotation
    rotate_backups(dir, &path)?;

    // 1. Serialize to string first
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| DeclarchError::SerializationError(format!("State serialization: {}", e)))?;

    // 2. Validate JSON is well-formed by parsing it back
    let _: State = serde_json::from_str(&content)
        .map_err(|e| DeclarchError::SerializationError(format!("Invalid JSON generated: {}", e)))?;

    // 3. Write to temp file
    let tmp_path = dir.join("state.tmp");
    let mut tmp_file = fs::File::create(&tmp_path).map_err(|e| DeclarchError::IoError {
        path: tmp_path.clone(),
        source: e,
    })?;

    tmp_file.write_all(content.as_bytes())?;
    tmp_file.sync_all()?;

    fs::rename(&tmp_path, &path).map_err(|e| DeclarchError::IoError {
        path: path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Save state with file locking to prevent concurrent access corruption
/// This is the preferred method for saving state in production
/// 
/// IMPORTANT: The lock is acquired at the START of the sync operation (not just during save)
/// to prevent concurrent modifications. Use `acquire_lock()` at the beginning of sync.
pub fn save_state_locked(state: &State, _lock: &StateLock) -> Result<()> {
    let path = get_state_path()?;
    let dir = path.parent().ok_or(DeclarchError::Other(
        "Could not determine state directory".into(),
    ))?;

    // Perform backup rotation (same as save_state)
    rotate_backups(dir, &path)?;

    // Serialize to string
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| DeclarchError::SerializationError(format!("State serialization: {}", e)))?;

    // Validate JSON
    let _: State = serde_json::from_str(&content)
        .map_err(|e| DeclarchError::SerializationError(format!("Invalid JSON generated: {}", e)))?;

    // Write to temp file
    let tmp_path = dir.join("state.tmp");
    let mut tmp_file = fs::File::create(&tmp_path).map_err(|e| DeclarchError::IoError {
        path: tmp_path.clone(),
        source: e,
    })?;

    tmp_file.write_all(content.as_bytes())?;
    tmp_file.sync_all()?;
    drop(tmp_file); // Close temp file before rename

    // Atomic rename
    fs::rename(&tmp_path, &path).map_err(|e| DeclarchError::IoError {
        path: path.clone(),
        source: e,
    })?;

    // Sync directory to ensure rename is persisted
    if let Ok(dir_file) = fs::File::open(dir) {
        if let Err(e) = dir_file.sync_all() {
            ui::warning(&format!("Failed to sync state directory: {}", e));
        }
    }

    // Lock is released when StateLock is dropped (RAII)
    Ok(())
}

pub fn init_state(hostname: String) -> Result<State> {
    let mut state = State::default();
    state.meta.hostname = hostname;
    save_state(&state)?;
    Ok(state)
}

/// Filesystem-based state store implementing the StateStore trait
pub struct FilesystemStateStore;

impl crate::traits::StateStore for FilesystemStateStore {
    fn load(&self) -> Result<State> {
        load_state()
    }

    fn save(&self, state: &State) -> Result<()> {
        // Trait implementation uses non-locking save
        // (locking is handled by caller when needed)
        save_state(state)
    }

    fn init(&self) -> Result<State> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());
        init_state(hostname)
    }
}
