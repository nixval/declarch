mod backup_ops;
mod load_recovery;
mod locking;
mod migration;
mod persist;

use crate::error::{DeclarchError, Result};
use crate::state::types::State;
use crate::utils::paths;
use load_recovery::load_state_from_path;
pub use locking::{StateLock, acquire_lock};
use migration::sanitize_state_in_place;
use persist::prepare_and_write_state;
use std::fs::{self};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const CURRENT_STATE_SCHEMA_VERSION: u8 = 3;

#[derive(Debug, Clone, Default)]
pub struct StateRepairReport {
    pub total_before: usize,
    pub total_after: usize,
    pub removed_empty_name: usize,
    pub removed_duplicates: usize,
    pub rekeyed_entries: usize,
    pub normalized_fields: usize,
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
        let canonical = crate::core::resolver::make_state_key(&crate::core::types::PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        });
        if key != &canonical {
            issues.push(format!(
                "Non-canonical state key: {} (expected {})",
                key, canonical
            ));
        }
    }

    // Check for future timestamps
    let now = SystemTime::now();
    if let Ok(last_sync) = pkg_state_timestamp(&state.meta.last_sync)
        && last_sync > now
    {
        issues.push("Last sync timestamp is in the future".to_string());
    }

    issues
}

/// Normalize and repair state package entries.
/// - Removes entries with empty config_name
/// - Removes duplicate backend+config_name signatures (keeps first)
/// - Rewrites keys to canonical "backend:config_name"
/// - Normalizes empty provides_name to config_name
pub fn repair_state_packages() -> Result<StateRepairReport> {
    let mut state = load_state()?;
    let report = sanitize_state_in_place(&mut state);
    if report.total_before != report.total_after
        || report.rekeyed_entries > 0
        || report.normalized_fields > 0
    {
        save_state(&state)?;
    }
    Ok(report)
}

fn pkg_state_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> Result<SystemTime> {
    let secs = dt.timestamp() as u64;
    Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

pub fn get_state_path() -> Result<PathBuf> {
    let state_file = paths::state_file()?;
    let state_dir = state_file
        .parent()
        .ok_or_else(|| DeclarchError::PathError("Could not determine state directory".into()))?;

    if !state_dir.exists() {
        fs::create_dir_all(state_dir).map_err(|e| DeclarchError::IoError {
            path: state_dir.to_path_buf(),
            source: e,
        })?;
    }

    Ok(state_file)
}

fn load_state_internal(strict_recovery: bool) -> Result<State> {
    let path = get_state_path()?;
    load_state_from_path(&path, strict_recovery)
}

pub fn load_state() -> Result<State> {
    load_state_internal(false)
}

/// Load state in strict mode.
///
/// In strict mode, fallback-to-default is disabled when the state file is unreadable/corrupted
/// and backup restore also fails. This is intended for high-risk mutating flows.
pub fn load_state_strict() -> Result<State> {
    load_state_internal(true)
}

pub fn save_state(state: &State) -> Result<()> {
    let path = get_state_path()?;
    prepare_and_write_state(state, &path, false)?;

    Ok(())
}

/// Save state with file locking to prevent concurrent access corruption
/// This is the preferred method for saving state in production
///
/// IMPORTANT: The lock is acquired at the START of the sync operation (not just during save)
/// to prevent concurrent modifications. Use `acquire_lock()` at the beginning of sync.
pub fn save_state_locked(state: &State, _lock: &StateLock) -> Result<()> {
    let path = get_state_path()?;
    prepare_and_write_state(state, &path, true)?;

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

#[cfg(test)]
mod tests;
