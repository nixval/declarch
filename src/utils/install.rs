use crate::error::{DeclarchError, Result};
use crate::ui as output;
use std::process::{Command, Stdio};

/// Install Soar package manager automatically
///
/// Attempts to install Soar using curl or wget, falling back
/// to manual installation instructions if both fail.
pub fn install_soar() -> Result<bool> {
    output::info("Attempting to install Soar...");

    // Check if Soar is already installed
    if is_soar_installed() {
        output::success("Soar is already installed");
        return Ok(true);
    }

    // Try curl first
    if let Ok(true) = try_install_with_curl() {
        // Verify installation
        if is_soar_installed() {
            output::success("Soar installed successfully via curl");
            return Ok(true);
        }
    }

    // Fallback to wget
    if let Ok(true) = try_install_with_wget() {
        // Verify installation
        if is_soar_installed() {
            output::success("Soar installed successfully via wget");
            return Ok(true);
        }
    }

    // Both failed
    output::error("Failed to install Soar automatically");
    output::warning("Please install Soar manually:");
    output::indent("https://github.com/pkgforge/soar#installation", 1);
    output::indent(
        "curl -fsSL https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh",
        1,
    );

    Ok(false)
}

/// Check if Soar is installed on the system
pub fn is_soar_installed() -> bool {
    std::path::Path::new("/usr/bin/soar").exists()
        || std::path::Path::new("/usr/local/bin/soar").exists()
        || Command::new("which")
            .arg("soar")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

/// Try installing Soar using curl
fn try_install_with_curl() -> Result<bool> {
    let curl_check = Command::new("which").arg("curl").output();

    if curl_check.map(|o| !o.status.success()).unwrap_or(true) {
        return Err(DeclarchError::Other("curl not found".into()));
    }

    output::indent("Using curl to download Soar installer...", 1);

    let status = Command::new("sh")
        .arg("-c")
        .arg("curl -fsSL https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: "curl".into(),
            reason: e.to_string(),
        })?;

    Ok(status.success())
}

/// Try installing Soar using wget
fn try_install_with_wget() -> Result<bool> {
    let wget_check = Command::new("which").arg("wget").output();

    if wget_check.map(|o| !o.status.success()).unwrap_or(true) {
        return Err(DeclarchError::Other("wget not found".into()));
    }

    output::indent("Using wget to download Soar installer...", 1);

    let status = Command::new("sh")
        .arg("-c")
        .arg("wget -qO- https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| DeclarchError::SystemCommandFailed {
            command: "wget".into(),
            reason: e.to_string(),
        })?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_soar_installed() {
        // This test just verifies the function doesn't panic
        let _installed = is_soar_installed();
    }

    #[test]
    fn test_curl_check() {
        // Verify curl check doesn't panic
        let result = try_install_with_curl();
        // We expect this to fail in most test environments
        // unless Soar and curl are actually available
        let _ = result;
    }

    #[test]
    fn test_wget_check() {
        // Verify wget check doesn't panic
        let result = try_install_with_wget();
        // We expect this to fail in most test environments
        let _ = result;
    }
}
