use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::utils::update_check::InstallOwner;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(super) fn build_windows_update_bootstrap_ps(version: &str) -> String {
    let script_ref = format!("v{}", version);
    let install_url = format!(
        "https://raw.githubusercontent.com/{}/{}/install.ps1",
        project_identity::REPO_SLUG,
        script_ref
    );
    let ps_inner = format!(
        "$ErrorActionPreference='Stop'; $u='{url}'; $s=(Invoke-WebRequest -UseBasicParsing -Uri $u).Content; $sb=[scriptblock]::Create($s); & $sb -Version '{version}' -Repo '{repo}'",
        url = install_url,
        version = version,
        repo = project_identity::REPO_SLUG
    );
    format!(
        "Start-Process -WindowStyle Hidden -FilePath powershell -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-Command',\"Start-Sleep -Seconds 2; {inner}\"",
        inner = ps_inner.replace('"', "\\\"")
    )
}

pub(super) fn normalize_requested_version(input: &str) -> Result<String> {
    let clean = input.trim().trim_start_matches('v');
    if clean.is_empty() {
        return Err(DeclarchError::Other(
            "Invalid version. Use semantic version like 0.8.2".to_string(),
        ));
    }

    if !clean.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(DeclarchError::Other(
            "Invalid version. Only digits and dots are allowed (example: 0.8.2)".to_string(),
        ));
    }
    Ok(clean.to_string())
}

pub(super) fn managed_update_hint(owner: &InstallOwner) -> String {
    match owner {
        InstallOwner::Pacman => {
            format!(
                "This {} installation is managed by pacman/AUR. Use: paru -Syu {}",
                project_identity::BINARY_NAME,
                project_identity::BINARY_NAME
            )
        }
        InstallOwner::Homebrew => {
            format!(
                "This {} installation is managed by Homebrew. Use: brew upgrade {}",
                project_identity::BINARY_NAME,
                project_identity::BINARY_NAME
            )
        }
        InstallOwner::Scoop => {
            format!(
                "This {} installation is managed by Scoop. Use: scoop update {}",
                project_identity::BINARY_NAME,
                project_identity::BINARY_NAME
            )
        }
        InstallOwner::Winget => {
            format!(
                "This {} installation is managed by Winget. Use: winget upgrade {}",
                project_identity::BINARY_NAME,
                project_identity::BINARY_NAME
            )
        }
        _ => {
            format!(
                "This {} installation is managed externally. Use your package manager to update.",
                project_identity::BINARY_NAME
            )
        }
    }
}
