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
        r#"// declarch configuration for {hostname}

meta {{
    title "{hostname} Setup"
    description "Base configuration"
}}

editor "nano"

imports {{
    "modules/base.kdl"
}}

pkg {{
    // Add packages here
    // Example:
    // aur {{ bat ripgrep }}
    // flatpak {{ org.gimp.GIMP }}
}}

excludes {{
    // Optional excluded packages
}}

backends {{
    // Initialize backends first, then import them here.
    // Example:
    // "backends/aur.kdl"
    // "backends/flatpak.kdl"
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

// Add your essential packages here
// Example:
// pkg {
//     backend-name {
//         bat         // Better cat with syntax highlighting
//         eza         // Better ls with colors
//         fd          // Better find command
//         ripgrep     // Faster grep alternative
//         zoxide      // Smart cd command
//         git         // Version control
//     }
// }
pkg {
    // Add packages after initializing your preferred backend
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

    #[test]
    fn test_default_host_uses_declarch_backends_block_only() {
        let tpl = default_host("laptop");
        assert!(tpl.contains("backends {"));
        assert!(!tpl.contains("backends.kdl"));
    }
}
