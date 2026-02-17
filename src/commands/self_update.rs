use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use crate::utils::update_check::{
    compare_versions, current_version, detect_install_owner, is_managed_by_package_manager,
    latest_version_live,
};

mod policy;
#[cfg(test)]
mod tests;
#[cfg(not(target_os = "windows"))]
mod unix_ops;

#[cfg(target_os = "windows")]
use policy::build_windows_update_bootstrap_ps;
use policy::{managed_update_hint, normalize_requested_version};
#[cfg(not(target_os = "windows"))]
use unix_ops::perform_self_update_unix;

pub struct SelfUpdateOptions {
    pub check: bool,
    pub version: Option<String>,
    pub yes: bool,
}

pub fn run(options: SelfUpdateOptions) -> Result<()> {
    let current = current_version();
    let target = match options.version {
        Some(version) => normalize_requested_version(&version)?,
        None => latest_version_live().ok_or_else(|| {
            DeclarchError::Other(
                "Unable to fetch latest release from GitHub. Check network and retry.".to_string(),
            )
        })?,
    };

    if options.check {
        if compare_versions(&target, &current).is_gt() {
            output::warning(&format!("Update available: {} -> {}", current, target));
        } else {
            output::success(&format!("Already up-to-date ({})", current));
        }
        return Ok(());
    }

    if !compare_versions(&target, &current).is_gt() {
        output::success(&format!("Already up-to-date ({})", current));
        return Ok(());
    }

    let owner = detect_install_owner();
    if is_managed_by_package_manager(&owner) {
        return Err(DeclarchError::Other(managed_update_hint(&owner)));
    }

    if !options.yes
        && !output::prompt_yes_no(&format!(
            "Update {} from {} to {}?",
            project_identity::BINARY_NAME,
            current,
            target
        ))
    {
        output::warning("Update cancelled.");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        perform_self_update_windows(&target)?;
        output::success(&format!(
            "Update to {} has been started in the background. Open a new terminal after it finishes.",
            target
        ));
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        perform_self_update_unix(&target)?;
    }

    output::success(&format!(
        "{} updated successfully.",
        project_identity::BINARY_NAME
    ));
    Ok(())
}

#[cfg(target_os = "windows")]
fn perform_self_update_windows(version: &str) -> Result<()> {
    use std::process::Command;

    let ps_bootstrap = build_windows_update_bootstrap_ps(version);

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_bootstrap,
        ])
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: "powershell".to_string(),
            reason: e.to_string(),
        })?;

    if !status.success() {
        return Err(DeclarchError::Other(
            "Failed to start Windows self-update process.".to_string(),
        ));
    }
    Ok(())
}
