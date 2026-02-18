use crate::utils::paths;
use crate::utils::update_check::fetcher;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct UpdateCache {
    pub(super) checked_at_unix: i64,
    pub(super) latest_version: String,
}

pub(super) fn get_latest_version_cached(ttl_secs: i64, timeout: Duration) -> Option<String> {
    let now = now_unix();
    let cached = read_cache();
    if let Some(cache) = cached.as_ref()
        && should_use_cache(now, cache.checked_at_unix, ttl_secs)
    {
        return Some(cache.latest_version.clone());
    }

    let fetched = fetcher::fetch_latest_version(timeout);
    if let Some(version) = pick_latest_version(now, ttl_secs, cached.as_ref(), fetched.as_deref()) {
        // Persist only when fetched version is newer than cache decision source.
        if cached
            .as_ref()
            .map(|c| c.latest_version.as_str() != version.as_str())
            .unwrap_or(true)
        {
            let _ = write_cache(&UpdateCache {
                checked_at_unix: now,
                latest_version: version.clone(),
            });
        }
        return Some(version);
    }

    None
}

pub(super) fn should_use_cache(now: i64, checked_at_unix: i64, ttl_secs: i64) -> bool {
    now.saturating_sub(checked_at_unix) <= ttl_secs
}

pub(super) fn pick_latest_version(
    now: i64,
    ttl_secs: i64,
    cached: Option<&UpdateCache>,
    fetched: Option<&str>,
) -> Option<String> {
    if let Some(cache) = cached
        && should_use_cache(now, cache.checked_at_unix, ttl_secs)
    {
        return Some(cache.latest_version.clone());
    }

    fetched
        .map(ToString::to_string)
        .or_else(|| cached.map(|c| c.latest_version.clone()))
}

fn cache_path() -> Option<PathBuf> {
    Some(paths::state_dir().ok()?.join("update-check-cache.json"))
}

fn read_cache() -> Option<UpdateCache> {
    let path = cache_path()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<UpdateCache>(&raw).ok()
}

fn write_cache(cache: &UpdateCache) -> Option<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok()?;
    }
    let raw = serde_json::to_string(cache).ok()?;
    fs::write(path, raw).ok()?;
    Some(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
