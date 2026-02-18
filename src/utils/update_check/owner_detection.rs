use crate::project_identity;
use crate::utils::update_check::InstallOwner;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn detect_install_owner() -> InstallOwner {
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

    if has_script_marker() {
        return InstallOwner::Script;
    }

    InstallOwner::Unknown
}

fn has_script_marker() -> bool {
    let Some(path) = install_marker_path() else {
        return false;
    };
    fs::read_to_string(path)
        .map(|content| install_channel_is_script_like(&content))
        .unwrap_or(false)
}

pub(super) fn install_channel_is_script_like(content: &str) -> bool {
    let normalized = content.to_ascii_lowercase();
    normalized.contains("\"channel\":\"script\"")
        || normalized.contains("\"channel\":\"curl\"")
        || normalized.contains("\"channel\":\"wget\"")
        || normalized.contains("\"channel\":\"manual\"")
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
        Some(
            path.join(project_identity::STABLE_PROJECT_ID)
                .join("install-channel.json"),
        )
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
