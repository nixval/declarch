use serde_json::Value;
use std::fs;
use std::time::SystemTime;
use anyhow::{Result, Context};

/// Load .state.json
pub fn load_state() -> Result<State> {
    let data = fs::read_to_string(".state.json")
        .context("Unable to read .state.json file")?;

    serde_json::from_str(&data)
        .context("Failed to parse .state.json. The file may be corrupted.")
}

/// Save .state.json
pub fn save_state(state: &State) -> Result<()> {
    let data = serde_json::to_string_pretty(state)
        .context("Failed to serialize state")?;

    fs::write(".state.json", data).context("Failed to write to .state.json.")
}

/// Backup .state.json
pub fn backup_state() -> Result<String> {
    let backup_file = format!(
        ".state-{}.json.bak",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    fs::copy(".state.json", &backup_file)
        .context("Failed to create backup of .state.json")?;
    Ok(backup_file)
}

/// Rollback .state.json from a backup
pub fn rollback_state(backup_file: &str) -> Result<()> {
    fs::copy(backup_file, ".state.json")
        .context("Failed to rollback .state.json from backup")
}
