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
        r#"// Host configuration for {hostname}

// === META ===
meta {{
    description "My {hostname} setup"
    author "nixval"
    version "1.0.0"
}}

// === IMPORTS ===
imports {{
    "modules/base.kdl"
    // Add more modules here
    // "modules/desktop.kdl"
    // "modules/development.kdl"
}}

// === PACKAGES ===
packages {{
    // Add AUR packages (default, Arch-only)
    // hyprland
    // waybar

    // Or use inline prefix syntax:
    // aur:waybar
    // soar:bat
    // flatpak:org.mozilla.firefox
}}

// === CONFLICTS ===
// conflicts {{
//     vim neovim
//     pipewire pulseaudio
// }}

// === BACKEND OPTIONS ===
// options:aur {{
//     noconfirm
// }}

// === ENVIRONMENT VARIABLES ===
// env EDITOR="nvim" VISUAL="nvim"
// env:aur MAKEFLAGS="-j4"

// === REPOSITORIES ===
// repos:aur {{
//     "https://aur.archlinux.org"
// }}

// === POLICY ===
// policy {{
//     protected {{
//         linux
//         systemd
//     }}
//     orphans "keep"
// }}

// === HOOKS ===
// hooks {{
//     post-sync {{
//         run "notify-send 'Packages updated'"
//         sudo-needed "systemctl restart gdm"
//     }}
// }}

excludes {{
    // Add packages to exclude
}}
"#,
        hostname = hostname
    )
}

/// Default module configuration template
pub fn default_module(name: &str) -> String {
    format!(
        r#"// Module: {name}

meta {{
    description "Packages for {name}"
    author "nixval"
}}

packages {{
    // Add packages here
    // zsh
    // git
}}
"#,
        name = name
    )
}

// --- NEW FUNCTION FOR INIT MODULE ---
/// Returns a specific template content based on the name (slug)
/// Contributors can add new recipes here!
pub fn get_template_by_name(name: &str) -> Option<String> {
    match name.to_lowercase().as_str() {
        "base" => Some(r#"// Base System Configuration
// Maintainer: nixval
// Description: Essential packages for any Linux system

meta {{
    description "Base system packages"
    author "nixval"
    version "1.0.0"
    tags "base" "essential"
}}

packages {{
    // Cross-distro essentials (Soar)
    bat         // Better cat with syntax highlighting
    exa         // Better ls with colors
    fd          // Better find command
    ripgrep     // Faster grep alternative
    zoxide      // Smart cd command

    // AUR packages (Arch Linux only)
    // zsh
    // fastfetch
}}

// Environment variables for this module
// env EDITOR="nvim"
"#.to_string()),

        "dev" => Some(r#"// Developer Tools
// Maintainer: Declarch Community

meta {{
    description "Development tools and IDEs"
    author "nixval"
}}

packages {{
    mise
}}
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
