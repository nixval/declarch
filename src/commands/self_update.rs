use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::update_check::{
    InstallOwner, compare_versions, current_version, detect_install_owner,
    is_managed_by_package_manager, latest_version_live,
};
#[cfg(not(target_os = "windows"))]
use reqwest::blocking::Client;
#[cfg(not(target_os = "windows"))]
use std::env;
#[cfg(not(target_os = "windows"))]
use std::fs;
#[cfg(not(target_os = "windows"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "windows"))]
use std::process::Command;
#[cfg(not(target_os = "windows"))]
use std::time::Duration;

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
        && !output::prompt_yes_no(&format!("Update declarch from {} to {}?", current, target))
    {
        output::warning("Update cancelled.");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        return Err(DeclarchError::Other(
            "Self-update is not supported on Windows yet. Use:\nirm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex"
                .to_string(),
        ));
    }

    #[cfg(not(target_os = "windows"))]
    {
        perform_self_update_unix(&target)?;
    }

    output::success("declarch updated successfully.");
    Ok(())
}

fn normalize_requested_version(input: &str) -> Result<String> {
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

fn managed_update_hint(owner: &InstallOwner) -> String {
    match owner {
        InstallOwner::Pacman => {
            "This declarch installation is managed by pacman/AUR. Use: paru -Syu declarch"
                .to_string()
        }
        InstallOwner::Homebrew => {
            "This declarch installation is managed by Homebrew. Use: brew upgrade declarch"
                .to_string()
        }
        InstallOwner::Scoop => {
            "This declarch installation is managed by Scoop. Use: scoop update declarch".to_string()
        }
        InstallOwner::Winget => {
            "This declarch installation is managed by Winget. Use: winget upgrade declarch"
                .to_string()
        }
        _ => {
            "This declarch installation is managed externally. Use your package manager to update."
                .to_string()
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn perform_self_update_unix(version: &str) -> Result<()> {
    let target = detect_target_triple()?;
    let asset = format!("declarch-{}.tar.gz", target);
    let base = format!(
        "https://github.com/nixval/declarch/releases/download/v{}",
        version
    );
    let asset_url = format!("{}/{}", base, asset);
    let checksums_url = format!("{}/checksums.txt", base);

    let tmp = env::temp_dir().join(format!("declarch-self-update-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;

    let archive_path = tmp.join(&asset);
    let checksums_path = tmp.join("checksums.txt");

    output::info(&format!("Downloading {}...", asset));
    download_file(&asset_url, &archive_path)?;
    output::info("Downloading checksums...");
    download_file(&checksums_url, &checksums_path)?;

    let expected = find_expected_checksum(&checksums_path, &asset)?;
    let actual = compute_sha256(&archive_path)?;
    verify_checksum(&asset, &expected, &actual)?;
    output::success("Checksum verification passed.");

    extract_archive(&archive_path, &tmp)?;
    let new_bin = tmp.join("declarch");
    if !new_bin.exists() {
        return Err(DeclarchError::Other(
            "Archive did not contain declarch binary".to_string(),
        ));
    }

    let current_exe = env::current_exe()
        .map_err(|e| DeclarchError::Other(format!("Unable to locate current executable: {}", e)))?;
    let backup = create_backup_binary(&current_exe)?;
    install_binary_with_elevation_if_needed(&new_bin, &current_exe)?;

    let installed = Command::new(&current_exe)
        .arg("--version")
        .output()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: format!("{} --version", current_exe.display()),
            reason: e.to_string(),
        })?;
    if !installed.status.success() {
        let _ = restore_from_backup(&backup, &current_exe);
        return Err(DeclarchError::Other(
            "Updated binary failed smoke check (--version). Rolled back to previous binary."
                .to_string(),
        ));
    }
    let _ = fs::remove_file(&backup);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn detect_target_triple() -> Result<&'static str> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        _ => Err(DeclarchError::Other(format!(
            "Self-update is not supported for this platform: {}/{}",
            env::consts::OS,
            env::consts::ARCH
        ))),
    }
}

#[cfg(not(target_os = "windows"))]
fn download_file(url: &str, path: &Path) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| DeclarchError::RemoteFetchError(e.to_string()))?;
    let response = client
        .get(url)
        .header("User-Agent", "declarch-cli")
        .send()
        .map_err(|e| DeclarchError::RemoteFetchError(e.to_string()))?;
    if !response.status().is_success() {
        return Err(DeclarchError::RemoteFetchError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }
    let body = response
        .bytes()
        .map_err(|e| DeclarchError::RemoteFetchError(e.to_string()))?;
    fs::write(path, &body)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn find_expected_checksum(checksums_path: &Path, asset_name: &str) -> Result<String> {
    let content = fs::read_to_string(checksums_path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((sha, file)) = parse_checksum_line(trimmed)
            && file == asset_name
        {
            return Ok(sha.to_string());
        }
    }
    Err(DeclarchError::Other(format!(
        "Could not find checksum entry for {}",
        asset_name
    )))
}

#[cfg(not(target_os = "windows"))]
fn parse_checksum_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let sha = parts.next()?;
    let file = parts.next()?;
    let file = file.strip_prefix('*').unwrap_or(file);
    Some((sha, file))
}

#[cfg(not(target_os = "windows"))]
fn compute_sha256(path: &Path) -> Result<String> {
    let out = Command::new("sha256sum").arg(path).output();
    if let Ok(output) = out
        && output.status.success()
    {
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(value) = line.split_whitespace().next() {
            return Ok(value.to_string());
        }
    }

    let out = Command::new("shasum")
        .arg("-a")
        .arg("256")
        .arg(path)
        .output();
    if let Ok(output) = out
        && output.status.success()
    {
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(value) = line.split_whitespace().next() {
            return Ok(value.to_string());
        }
    }

    Err(DeclarchError::Other(
        "No SHA256 tool found (need sha256sum or shasum).".to_string(),
    ))
}

#[cfg(not(target_os = "windows"))]
fn extract_archive(archive: &Path, out_dir: &Path) -> Result<()> {
    let status = Command::new("tar")
        .arg("xzf")
        .arg(archive)
        .arg("-C")
        .arg(out_dir)
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: format!("tar xzf {} -C {}", archive.display(), out_dir.display()),
            reason: e.to_string(),
        })?;
    if !status.success() {
        return Err(DeclarchError::Other(
            "Failed to extract update archive".to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn verify_checksum(asset: &str, expected: &str, actual: &str) -> Result<()> {
    if expected.eq_ignore_ascii_case(actual) {
        return Ok(());
    }

    Err(DeclarchError::Other(format!(
        "Checksum verification failed for {} (expected {}, got {})",
        asset, expected, actual
    )))
}

#[cfg(not(target_os = "windows"))]
fn create_backup_binary(current_exe: &Path) -> Result<PathBuf> {
    let backup = current_exe.with_extension("declarch.backup");
    fs::copy(current_exe, &backup)?;
    Ok(backup)
}

#[cfg(not(target_os = "windows"))]
fn restore_from_backup(backup: &Path, current_exe: &Path) -> Result<()> {
    let dest_dir = current_exe.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot resolve install directory for {}",
            current_exe.display()
        ))
    })?;

    if dest_dir_is_writable(dest_dir) {
        fs::copy(backup, current_exe)?;
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("install")
        .arg("-m")
        .arg("755")
        .arg(backup)
        .arg(current_exe)
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: format!(
                "sudo install -m 755 {} {}",
                backup.display(),
                current_exe.display()
            ),
            reason: e.to_string(),
        })?;
    if !status.success() {
        return Err(DeclarchError::Other(
            "Failed to rollback previous binary after update failure".to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn install_binary_with_elevation_if_needed(new_bin: &Path, current_exe: &Path) -> Result<()> {
    let dest_dir = current_exe.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot resolve install directory for {}",
            current_exe.display()
        ))
    })?;

    if dest_dir_is_writable(dest_dir) {
        fs::copy(new_bin, current_exe)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(current_exe, fs::Permissions::from_mode(0o755))?;
        }
        update_decl_symlink(dest_dir, current_exe)?;
        return Ok(());
    }

    let install_status = Command::new("sudo")
        .arg("install")
        .arg("-m")
        .arg("755")
        .arg(new_bin)
        .arg(current_exe)
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: format!(
                "sudo install -m 755 {} {}",
                new_bin.display(),
                current_exe.display()
            ),
            reason: e.to_string(),
        })?;
    if !install_status.success() {
        return Err(DeclarchError::Other(
            "Failed to install updated binary with sudo".to_string(),
        ));
    }

    update_decl_symlink(dest_dir, current_exe)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn dest_dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(".declarch-write-check");
    let created = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&probe)
        .is_ok();
    let _ = fs::remove_file(probe);
    created
}

#[cfg(not(target_os = "windows"))]
fn update_decl_symlink(dir: &Path, target: &Path) -> Result<()> {
    let decl = dir.join("decl");
    if decl.exists() && !decl.is_symlink() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        if decl.exists() {
            let _ = fs::remove_file(&decl);
        }
        std::os::unix::fs::symlink(target, &decl)?;
    }
    Ok(())
}

#[cfg(test)]
#[cfg(not(target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn parse_checksum_line_supports_coreutils_and_bsd_style() {
        assert_eq!(
            parse_checksum_line("abc123  declarch-x86_64-unknown-linux-gnu.tar.gz"),
            Some(("abc123", "declarch-x86_64-unknown-linux-gnu.tar.gz"))
        );
        assert_eq!(
            parse_checksum_line("abc123 *declarch-x86_64-unknown-linux-gnu.tar.gz"),
            Some(("abc123", "declarch-x86_64-unknown-linux-gnu.tar.gz"))
        );
    }

    #[test]
    fn verify_checksum_detects_mismatch() {
        let result = verify_checksum(
            "declarch-x86_64-unknown-linux-gnu.tar.gz",
            "abc123",
            "def456",
        );
        assert!(result.is_err());
    }
}
