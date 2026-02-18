use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui;
use fs2::FileExt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

const LOCK_TIMEOUT_SECONDS: u64 = 300;

pub struct StateLock {
    _file: std::fs::File,
    path: PathBuf,
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire_lock() -> Result<StateLock> {
    let path = super::get_state_path()?;
    acquire_lock_for_state_path(path)
}

fn acquire_lock_for_state_path(path: PathBuf) -> Result<StateLock> {
    let dir = path
        .parent()
        .ok_or_else(|| DeclarchError::Other("Could not determine state directory".into()))?;
    let lock_path = dir.join("state.lock");

    if lock_path.exists() {
        let metadata = fs::metadata(&lock_path)?;
        let age_secs = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .map_or(0, |age| age.as_secs());

        let existing_file = OpenOptions::new().write(true).open(&lock_path)?;

        match existing_file.try_lock_exclusive() {
            Ok(()) => {
                if age_secs > LOCK_TIMEOUT_SECONDS {
                    ui::warning("Removing stale lock file (not actively locked)");
                }
                let _ = fs::remove_file(&lock_path);
            }
            Err(_) => {
                let age_hint = if age_secs > LOCK_TIMEOUT_SECONDS {
                    format!(
                        "\nLock appears older than {} seconds but is still actively locked.",
                        LOCK_TIMEOUT_SECONDS
                    )
                } else {
                    String::new()
                };
                return Err(DeclarchError::Other(format!(
                    "Another {} process is currently running.\n\
                     Lock file: {}{}\n\
                     Wait for it to complete, or delete the lock file if you're sure no other process is running.",
                    project_identity::BINARY_NAME,
                    lock_path.display(),
                    age_hint
                )));
            }
        }
    }

    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .map_err(|e| DeclarchError::IoError {
            path: lock_path.clone(),
            source: e,
        })?;

    lock_file
        .lock_exclusive()
        .map_err(|e| DeclarchError::Other(format!("Failed to lock state file: {}", e)))?;

    let pid = std::process::id();
    let _ = writeln!(&lock_file, "{}", pid);

    Ok(StateLock {
        _file: lock_file,
        path: lock_path,
    })
}

#[cfg(test)]
mod tests {
    use super::acquire_lock_for_state_path;
    use tempfile::tempdir;

    #[test]
    fn lock_contention_returns_error() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state.json");

        let _lock = acquire_lock_for_state_path(state_path.clone()).expect("first lock");
        match acquire_lock_for_state_path(state_path) {
            Ok(_) => panic!("second lock should fail"),
            Err(err) => assert!(err.to_string().contains("currently running")),
        }
    }
}
