/// Default global configuration
pub fn default_config() -> String {
    r#"// Global declarch configuration
aur_helper "paru"
"#
    .to_string()
}

/// Default host configuration template
pub fn default_host(hostname: &str) -> String {
    format!(
        r#"// Host configuration for {}
description "My {} setup"

// imports {{
//     "modules/base.kdl"
// }}

packages {{
    // Add packages here
    // git
    // vim
    // flatpak:org.mozilla.firefox
}}

excludes {{
    // Add packages to exclude
}}
"#,
        hostname, hostname
    )
}

/// Default module configuration template
pub fn default_module(name: &str) -> String {
    format!(
        r#"// Module: {}
description "Packages for {}"

packages {{
    // zsh
    // git
}}
"#,
        name, name
    )
}
// --- NEW FUNCTION FOR INIT MODULE ---
/// Returns a specific template content based on the name (slug)
/// Contributors can add new recipes here!
pub fn get_template_by_name(name: &str) -> Option<String> {
    match name.to_lowercase().as_str() {
        "hyprland" => Some(r#"// Hyprland Desktop Environment
// Maintainer: Declarch Community

packages {
    // Core
    hyprland
    hyprlock
    hypridle
    hyprpaper
    xdg-desktop-portal-hyprland
    
    // UI/UX
    waybar
    rofi-wayland
    dunst
    kitty
    
    // Qt/GTK Theming
    qt5-wayland
    qt6-wayland
    nwg-look
}
"#.to_string()),

        "gaming" => Some(r#"// Gaming Suite
// Maintainer: Declarch Community

packages {
    steam
    gamemode
    mangohud
    lutris
    
    // Wine dependencies
    wine
    winetricks
    
    // Common libs
    lib32-mesa
    lib32-gcc-libs
    lib32-glibc
}
"#.to_string()),

        "dev" => Some(r#"// Developer Tools
// Maintainer: Declarch Community

packages {
    git
    base-devel
    neovim
    lazygit
    ripgrep
    fd
    eza
    bat
    jq
}
"#.to_string()),

        _ => None, // Not found, will use generic default_module()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_not_empty() {
        assert!(!default_config().is_empty());
        assert!(!default_host("laptop").is_empty());
        assert!(!default_module("tools").is_empty());
    }
}
