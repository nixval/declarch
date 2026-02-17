use crate::project_identity;
use crate::utils::paths;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_TTL_SECS: i64 = 24 * 60 * 60;
const HTTP_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOwner {
    Pacman,
    Homebrew,
    Scoop,
    Winget,
    Script,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct UpdateHint {
    pub current: String,
    pub latest: String,
    pub owner: InstallOwner,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCache {
    checked_at_unix: i64,
    latest_version: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn is_managed_by_package_manager(owner: &InstallOwner) -> bool {
    matches!(
        owner,
        InstallOwner::Pacman | InstallOwner::Homebrew | InstallOwner::Scoop | InstallOwner::Winget
    )
}

pub fn detect_install_owner() -> InstallOwner {
    if has_script_marker() {
        return InstallOwner::Script;
    }

    let exe_path = env::current_exe().ok();

    #[cfg(target_os = "linux")]
    {
        if is_owned_by_pacman(exe_path.as_deref()) {
            return InstallOwner::Pacman;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if is_homebrew_managed(exe_path.as_deref()) {
            return InstallOwner::Homebrew;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if is_scoop_managed(exe_path.as_deref()) {
            return InstallOwner::Scoop;
        }
        if is_winget_managed(exe_path.as_deref()) {
            return InstallOwner::Winget;
        }
    }

    InstallOwner::Unknown
}

pub fn update_hint_cached() -> Option<UpdateHint> {
    let current = current_version();
    let latest = get_latest_version_cached()?;
    if compare_versions(&latest, &current).is_gt() {
        Some(UpdateHint {
            current,
            latest,
            owner: detect_install_owner(),
        })
    } else {
        None
    }
}

pub fn latest_version_live() -> Option<String> {
    fetch_latest_version(Duration::from_secs(HTTP_TIMEOUT_SECS))
}

fn get_latest_version_cached() -> Option<String> {
    let now = now_unix();
    if let Some(cache) = read_cache()
        && now.saturating_sub(cache.checked_at_unix) <= CACHE_TTL_SECS
    {
        return Some(cache.latest_version);
    }

    match fetch_latest_version(Duration::from_secs(HTTP_TIMEOUT_SECS)) {
        Some(version) => {
            let _ = write_cache(&UpdateCache {
                checked_at_unix: now,
                latest_version: version.clone(),
            });
            Some(version)
        }
        None => read_cache().map(|c| c.latest_version),
    }
}

fn fetch_latest_version(timeout: Duration) -> Option<String> {
    let client = Client::builder().timeout(timeout).build().ok()?;

    let response = client
        .get(project_identity::github_latest_release_api())
        .header(
            "User-Agent",
            format!("{}-cli", project_identity::BINARY_NAME),
        )
        .send()
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body = response.text().ok()?;
    let release: GitHubRelease = serde_json::from_str(&body).ok()?;
    let tag = release.tag_name.trim();
    let normalized = tag.strip_prefix('v').unwrap_or(tag).to_string();
    if version_tuple(&normalized).is_some() {
        Some(normalized)
    } else {
        None
    }
}

pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    match (version_tuple(a), version_tuple(b)) {
        (Some(va), Some(vb)) => va.cmp(&vb),
        _ => a.cmp(b),
    }
}

fn version_tuple(input: &str) -> Option<(u64, u64, u64)> {
    let core = input.trim().trim_start_matches('v');
    let core = core.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    Some((major, minor, patch))
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

fn has_script_marker() -> bool {
    let Some(path) = install_marker_path() else {
        return false;
    };
    fs::read_to_string(path)
        .map(|content| {
            let normalized = content.to_ascii_lowercase();
            normalized.contains("\"channel\":\"script\"")
                || normalized.contains("\"channel\":\"curl\"")
                || normalized.contains("\"channel\":\"wget\"")
                || normalized.contains("\"channel\":\"manual\"")
        })
        .unwrap_or(false)
}

fn install_marker_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local = env::var_os("LOCALAPPDATA")?;
        return Some(
            PathBuf::from(local)
                .join(project_identity::STABLE_PROJECT_ID)
                .join("install-channel.json"),
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        let path = if let Some(state_home) = env::var_os("XDG_STATE_HOME") {
            PathBuf::from(state_home)
        } else {
            let home = env::var_os("HOME")?;
            PathBuf::from(home).join(".local").join("state")
        };
        return Some(
            path.join(project_identity::STABLE_PROJECT_ID)
                .join("install-channel.json"),
        );
    }
}

fn is_owned_by_pacman(exe_path: Option<&Path>) -> bool {
    let Some(path) = exe_path else {
        return false;
    };
    let output = Command::new("pacman").arg("-Qo").arg(path).output();
    matches!(output, Ok(out) if out.status.success())
}

#[cfg(target_os = "macos")]
fn is_homebrew_managed(exe_path: Option<&Path>) -> bool {
    let Some(path) = exe_path else {
        return false;
    };

    let path_s = path.to_string_lossy().to_ascii_lowercase();
    if path_s.contains("/cellar/") || path_s.contains("/homebrew/") {
        return true;
    }

    let output = Command::new("brew")
        .arg("list")
        .arg("--versions")
        .arg(project_identity::BINARY_NAME)
        .output();
    matches!(output, Ok(out) if out.status.success())
}

#[cfg(target_os = "windows")]
fn is_scoop_managed(exe_path: Option<&Path>) -> bool {
    let Some(path) = exe_path else {
        return false;
    };
    path.to_string_lossy()
        .to_ascii_lowercase()
        .contains(&format!(
            "\\scoop\\apps\\{}\\",
            project_identity::BINARY_NAME
        ))
}

#[cfg(target_os = "windows")]
fn is_winget_managed(exe_path: Option<&Path>) -> bool {
    let Some(path) = exe_path else {
        return false;
    };
    let s = path.to_string_lossy().to_ascii_lowercase();
    s.contains("\\windowsapps\\") || s.contains("\\winget\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_works_for_semver() {
        assert!(compare_versions("0.8.2", "0.8.1").is_gt());
        assert!(compare_versions("0.8.1", "0.8.1").is_eq());
        assert!(compare_versions("0.8.0", "0.8.1").is_lt());
    }

    #[test]
    fn parses_prefixed_versions() {
        assert_eq!(version_tuple("v0.8.1"), Some((0, 8, 1)));
        assert_eq!(version_tuple("0.8.1-beta.1"), Some((0, 8, 1)));
    }
}
