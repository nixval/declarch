use crate::error::{DeclarchError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache duration in seconds (5 minutes)
const CACHE_TTL: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: String,
    pub timestamp: u64,
}

impl CacheEntry {
    pub fn new(data: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self { data, timestamp }
    }

    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.timestamp) < CACHE_TTL
    }
}

/// Get the cache directory path
fn get_cache_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "declarch", "declarch")
        .ok_or_else(|| DeclarchError::Other("Could not determine cache directory".into()))?;

    let cache_dir = proj_dirs.cache_dir().to_path_buf();

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| DeclarchError::IoError {
            path: cache_dir.clone(),
            source: e,
        })?;
    }

    Ok(cache_dir)
}

/// Get cached data for a given key
pub fn get(key: &str) -> Option<String> {
    let cache_dir = get_cache_dir().ok()?;
    let cache_path = cache_dir.join(format!("{}.json", sanitize_key(key)));

    if !cache_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&cache_path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;

    if entry.is_valid() {
        Some(entry.data)
    } else {
        // Remove expired cache entry
        let _ = fs::remove_file(&cache_path);
        None
    }
}

/// Set cached data for a given key
pub fn set(key: &str, data: &str) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_path = cache_dir.join(format!("{}.json", sanitize_key(key)));

    let entry = CacheEntry::new(data.to_string());
    let json = serde_json::to_string_pretty(&entry)?;

    fs::write(&cache_path, json).map_err(|e| DeclarchError::IoError {
        path: cache_path,
        source: e,
    })?;

    Ok(())
}

/// Clear all cached data
pub fn clear() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    for entry in fs::read_dir(&cache_dir).map_err(|e| DeclarchError::IoError {
        path: cache_dir.clone(),
        source: e,
    })? {
        let entry = entry.map_err(|e| DeclarchError::IoError {
            path: cache_dir.clone(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) == Some("json") {
            let _ = fs::remove_file(&path);
        }
    }

    Ok(())
}

/// Sanitize cache key to be safe for filenames
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_is_valid() {
        let entry = CacheEntry::new("test data".to_string());
        assert!(entry.is_valid());
    }

    #[test]
    fn test_cache_entry_expires() {
        let mut entry = CacheEntry::new("test data".to_string());
        // Simulate old entry
        entry.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - CACHE_TTL
            - 1;
        assert!(!entry.is_valid());
    }

    #[test]
    fn test_sanitize_key() {
        assert_eq!(sanitize_key("test-key"), "test-key");
        assert_eq!(sanitize_key("test/key"), "test_key");
        assert_eq!(sanitize_key("test key"), "test_key");
    }
}
