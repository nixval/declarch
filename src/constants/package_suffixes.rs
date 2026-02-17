/// Common package suffix patterns for variant matching
///
/// Used for:
/// - Switch command (hyprland → hyprland-git)
/// - Sync command (variant detection for hooks)
/// - Package name resolution
///
/// Example matching:
/// - "hyprland" → match if package is "hyprland-git"
/// - "ripgrep" → match if package is "ripgrep-bin"
pub const VARIANTS: &[&str] = &[
    "-git", "-hg", "-nightly", "-daily", "-beta", "-alpha", "-bin", "-binary", "-minimal", "-lite",
    "-full",
];

/// Build suffix patterns for package switching
///
/// Creates all possible variant combinations for a base package name.
/// Example: "hyprland" → ["hyprland-git", "hyprland-nightly", ...]
pub fn build_variants(base_name: &str) -> Vec<String> {
    VARIANTS
        .iter()
        .map(|suffix| format!("{}{}", base_name, suffix))
        .collect()
}

/// Check if a package name is a variant of base package
///
/// # Example
/// ```
/// # use declarch::constants::package_suffixes::is_variant;
/// // hyprland-git is a variant of hyprland
/// assert!(is_variant("hyprland-git", "hyprland")); // true
/// // hyprland is NOT a variant of hyprland-git (reverse is false)
/// assert!(!is_variant("hyprland", "hyprland-git")); // false
/// assert!(!is_variant("bat", "exa"));               // false
/// ```
pub fn is_variant(package_name: &str, base_name: &str) -> bool {
    if package_name == base_name {
        return false;
    }

    // Remove suffix to check if matches base
    for suffix in VARIANTS {
        if package_name == format!("{}{}", base_name, suffix) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests;
