use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use reqwest::blocking::Client;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub(super) fn perform_self_update_unix(version: &str) -> Result<()> {
    let target = detect_target_triple()?;
    let asset = format!(
        "{}-{}.tar.gz",
        project_identity::RELEASE_ASSET_PREFIX,
        target
    );
    let base = project_identity::release_download_base_url(version);
    let asset_url = format!("{}/{}", base, asset);
    let checksums_url = format!("{}/checksums.txt", base);

    let tmp = env::temp_dir().join(format!(
        "{}-self-update-{}",
        project_identity::STABLE_PROJECT_ID,
        std::process::id()
    ));
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
    let new_bin = tmp.join(project_identity::BINARY_NAME);
    if !new_bin.exists() {
        return Err(DeclarchError::Other(format!(
            "Archive did not contain {} binary",
            project_identity::BINARY_NAME
        )));
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

fn download_file(url: &str, path: &Path) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| DeclarchError::RemoteFetchError(e.to_string()))?;
    let response = client
        .get(url)
        .header(
            "User-Agent",
            format!("{}-cli", project_identity::BINARY_NAME),
        )
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

pub(super) fn parse_checksum_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let sha = parts.next()?;
    let file = parts.next()?;
    let file = file.strip_prefix('*').unwrap_or(file);
    Some((sha, file))
}

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

pub(super) fn verify_checksum(asset: &str, expected: &str, actual: &str) -> Result<()> {
    if expected.eq_ignore_ascii_case(actual) {
        return Ok(());
    }

    Err(DeclarchError::Other(format!(
        "Checksum verification failed for {} (expected {}, got {})",
        asset, expected, actual
    )))
}

fn create_backup_binary(current_exe: &Path) -> Result<PathBuf> {
    let backup = current_exe.with_extension(format!("{}.backup", project_identity::BINARY_NAME));
    fs::copy(current_exe, &backup)?;
    Ok(backup)
}

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

fn dest_dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(
        ".{}-write-check",
        project_identity::STABLE_PROJECT_ID
    ));
    let created = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&probe)
        .is_ok();
    let _ = fs::remove_file(probe);
    created
}

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
