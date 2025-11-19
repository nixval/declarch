/// Validate if flatpak is installed
pub fn validate_flatpak_installed() -> Result<()> {
    if is_command_found("flatpak") {
        Ok(())
    } else {
        Err(anyhow!("Flatpak is not installed. Please install 'flatpak' to proceed."))
    }
}

fn is_command_found(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map_or(false, |o| o.status.success())
}
