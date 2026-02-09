/// Default global configuration
pub fn default_config() -> String {
    r#"// Global declarch configuration
// Add global settings here
"#
    .to_string()
}

/// Default host configuration template
pub fn default_host(hostname: &str) -> String {
    format!(
        r#"// Host configuration for {hostname}

// === META ===
meta {{
    title "{hostname} Setup"
    description "My {hostname} setup"
    author "nixval"
    version "1.0.0"
}}

// === IMPORTS ===
imports {{
    "backends.kdl"
    "modules/base.kdl"
    // Add more modules here
    // "modules/desktop.kdl"
    // "modules/development.kdl"
}}

// === PACKAGES ===
pkg {{
    // Uncomment packages to install them
    // Example packages:
    // hyprland
    // waybar
    // bat
}}

// === CONFLICTS ===
// conflicts {{
//     vim neovim
//     pipewire pulseaudio
// }}

// === BACKEND OPTIONS ===
// options:paru {{
//     noconfirm
// }}

// === ENVIRONMENT VARIABLES ===
// env EDITOR="nvim" VISUAL="nvim"
// env:paru MAKEFLAGS="-j4"

// === REPOSITORIES ===
// repos:paru {{
//     "https://aur.archlinux.org"
// }}

// === POLICY ===
// policy {{
//     protected {{
//         linux
//         systemd
//     }}
//     orphans "ask"
// }}

// === HOOKS ===
// Note: Hooks are disabled by default for security.
// Use --hooks flag to enable: declarch sync --hooks
//
// on-sync "notify-send 'Packages updated'"
// on-sync-sudo "systemctl restart gdm"

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
    title "{name} Module"
    description "Packages for {name}"
    author "nixval"
}}

pkg {{
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
        "base" => Some(
            r#"// Base System Configuration
// Maintainer: nixval
// Description: Essential packages for any Linux system

meta {
    title "Base System"
    description "Essential packages for any Linux system"
    author "nixval"
    version "1.0.0"
    tags "base" "essential"
}

// Essential packages for modern terminal experience
pkg {
    aur {
        bat         // Better cat with syntax highlighting
        eza         // Better ls with colors (modern replacement for exa)
        fd          // Better find command
        ripgrep     // Faster grep alternative
        zoxide      // Smart cd command
        git         // Version control
        curl        // Data transfer tool
        wget        // Data transfer tool
    }
}
"#
            .to_string(),
        ),

        "dev" => Some(
            r#"// Developer Tools
// Maintainer: Declarch Community

meta {
    title "Development Tools"
    description "Development tools and utilities"
    author "nixval"
    version "1.0.0"
    tags "development" "tools"
}

pkg {
    // mise        // Version manager for various tools
    // lazydocker  // Docker TUI
}
"#
            .to_string(),
        ),

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
