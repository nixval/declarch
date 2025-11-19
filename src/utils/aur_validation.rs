/// Validate if an AUR helper (paru or yay) is available
pub fn validate_aur_helper() -> Result<String> {
    // Default to paru, but check yay as fallback
    if is_command_found("paru") {
        Ok("paru".to_string())
    } else if is_command_found("yay") {
        Ok("yay".to_string())
    } else {
        Err(anyhow!("No AUR helper found. Please install 'paru' or 'yay'."))
    }
}

/// Check if a command exists in the system
fn is_command_found(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map_or(false, |o| o.status.success())
}
