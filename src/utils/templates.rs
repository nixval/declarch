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

/// Default base module
pub fn default_base_module() -> String {
    r#"// Base system packages
description "Essential system packages"

packages {
    zsh
    git
    curl
    wget
    vim
    base-devel
}
"#
    .to_string()
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
