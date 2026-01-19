use kdl::{KdlDocument, KdlNode};
use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RawConfig {
    pub imports: Vec<String>,
    /// Packages from AUR (Arch Linux specific)
    /// Syntax: packages { ... } or packages:aur { ... }
    pub packages: Vec<String>,
    /// Packages from Soar registry (cross-distro static binaries)
    /// Syntax: packages:soar { ... }
    pub soar_packages: Vec<String>,
    /// Flatpak packages
    /// Syntax: packages:flatpak { ... } or flatpak-packages { ... }
    pub flatpak_packages: Vec<String>,
    pub excludes: Vec<String>,
    /// Package aliases: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub aliases: HashMap<String, String>,
    /// Editor to use for edit command
    /// Syntax: editor "nvim" or editor nvim
    pub editor: Option<String>,
}

pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse().map_err(|e: kdl::KdlError| {
        // Provide more helpful error messages for common KDL syntax issues
        let err_msg = e.to_string();
        let hint = if err_msg.contains("unexpected token") {
            "\nHint: Check for missing quotes, unmatched brackets, or invalid characters."
        } else if err_msg.contains("unexpected end of file") {
            "\nHint: You might be missing a closing brace '}' or parenthesis."
        } else if err_msg.contains("expected") {
            "\nHint: Check that your KDL syntax follows the format: node-name \"value\" { ... }"
        } else {
            ""
        };

        crate::error::DeclarchError::ConfigError(format!("KDL parsing error: {}{}", err_msg, hint))
    })?;

    let mut config = RawConfig {
        imports: vec![],
        packages: vec![],
        soar_packages: vec![],
        flatpak_packages: vec![],
        excludes: vec![],
        aliases: HashMap::new(),
        editor: None,
    };

    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            "import" | "imports" => {
                extract_strings(node, &mut config.imports);
            },
            "exclude" | "excludes" => {
                extract_mixed_values(node, &mut config.excludes);
            },
            "aliases-pkg" | "alias-pkg" => {
                extract_aliases(node, &mut config.aliases);
            },
            "editor" => {
                // Extract editor from first string argument
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string() {
                        config.editor = Some(val.to_string());
                    }
            },
            "description" => {
                // No-op, just ignore description nodes
            },
            // Parse packages with new flexible syntax
            name if name.starts_with("packages") => {
                parse_packages_node(node, &mut config)?;
            },
            // Legacy syntax support (with deprecation warning in the future)
            "aur-packages" | "aur-package" => {
                extract_mixed_values(node, &mut config.packages);
            },
            "soar-packages" | "soar-package" => {
                extract_mixed_values(node, &mut config.soar_packages);
            },
            "flatpak-packages" | "flatpak-package" => {
                extract_mixed_values(node, &mut config.flatpak_packages);
            },
            _ => {}
        }
    }

    Ok(config)
}

/// Parse packages node with flexible syntax
///
/// Supported syntaxes:
/// 1. packages { hyprland waybar }  → AUR packages (default)
/// 2. packages:aur { hyprland }  → AUR packages (explicit)
/// 3. packages:soar { bat exa }  → Soar packages
/// 4. packages:flatpak { com.spotify.Client }  → Flatpak packages
/// 5. packages { bat aur { hyprland } flatpak { com.spotify.Client } }  → Mixed
fn parse_packages_node(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    let node_name = node.name().value();

    // Check for colon syntax: packages:soar, packages:flatpak, packages:aur
    if let Some((_, backend)) = node_name.split_once(':') {
        let target = match backend {
            "aur" => &mut config.packages,
            "flatpak" => &mut config.flatpak_packages,
            "soar" | "app" => &mut config.soar_packages,
            _ => {
                // Unknown backend, treat as default (AUR)
                &mut config.packages
            }
        };
        extract_mixed_values(node, target);
        return Ok(());
    }

    // No colon syntax - check for embedded children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            match child_name {
                "aur" => {
                    extract_mixed_values(child, &mut config.packages);
                },
                "flatpak" => {
                    extract_mixed_values(child, &mut config.flatpak_packages);
                },
                "soar" | "app" => {
                    extract_mixed_values(child, &mut config.soar_packages);
                },
                // Unknown child name - treat as package name (AUR, the default)
                _ => {
                    config.packages.push(child_name.to_string());
                    // Also check for string arguments
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            config.packages.push(val.to_string());
                        }
                    }
                }
            }
        }
    }

    // Also extract direct string arguments (default to AUR)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            config.packages.push(val.to_string());
        }
    }

    Ok(())
}

fn extract_mixed_values(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                 if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

fn extract_strings(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

/// Extract package aliases from KDL node
///
/// Supported syntax:
/// ```kdl
/// aliases-pkg pipewire pipewire-jack2
/// aliases-pkg {
///     python-poetry python-poetry-core
///     firefox firefox-beta
/// }
/// ```
fn extract_aliases(node: &KdlNode, target: &mut HashMap<String, String>) {
    // Case 1: Inline format: aliases-pkg name1 name2
    let entries: Vec<_> = node.entries().iter()
        .filter_map(|e| e.value().as_string())
        .collect();

    if entries.len() == 2 {
        // aliases-pkg config_name actual_name
        target.insert(entries[0].to_string(), entries[1].to_string());
    }

    // Case 2: Children format
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_entries: Vec<_> = child.entries().iter()
                .filter_map(|e| e.value().as_string())
                .collect();

            if child_entries.len() == 2 {
                target.insert(child_entries[0].to_string(), child_entries[1].to_string());
            } else if child_entries.len() == 1 {
                // Support: node-name target-name
                let config_name = child.name().value();
                target.insert(config_name.to_string(), child_entries[0].to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aliases_inline() {
        let kdl = r#"
            aliases-pkg pipewire pipewire-jack2
            aliases-pkg python-poetry python-poetry-core
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
        assert_eq!(config.aliases.get("python-poetry"), Some(&"python-poetry-core".to_string()));
    }

    #[test]
    fn test_parse_aliases_block() {
        let kdl = r#"
            aliases-pkg {
                pipewire pipewire-jack2
                python-poetry python-poetry-core
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
        assert_eq!(config.aliases.get("python-poetry"), Some(&"python-poetry-core".to_string()));
    }

    #[test]
    fn test_parse_mixed_config() {
        let kdl = r#"
            packages {
                neovim
                hyprland
            }

            aliases-pkg pipewire pipewire-jack2

            excludes bad-package
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.aliases.len(), 1);
        assert_eq!(config.excludes.len(), 1);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
    }

    #[test]
    fn test_parse_empty_aliases() {
        let kdl = r#"
            packages neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_parse_aur_packages() {
        let kdl = r#"
            aur-packages {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));
    }

    #[test]
    fn test_parse_default_packages() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                swww
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));
        assert!(config.packages.contains(&"swww".to_string()));
    }

    #[test]
    fn test_parse_soar_packages() {
        let kdl = r#"
            soar-packages {
                bat
                exa
                ripgrep
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 3);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));
        assert!(config.soar_packages.contains(&"ripgrep".to_string()));
    }

    #[test]
    fn test_parse_flatpak_packages() {
        let kdl = r#"
            flatpak-packages {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
        assert!(config.flatpak_packages.contains(&"org.mozilla.firefox".to_string()));
    }

    #[test]
    fn test_parse_cross_distro_config() {
        let kdl = r#"
            // Cross-distro configuration example

            // AUR packages (default, Arch-only)
            packages {
                hyprland
                waybar
                swww
            }

            // Soar packages (cross-distro static binaries)
            soar-packages {
                bat
                exa
                fd
                ripgrep
            }

            // Flatpak packages (cross-distro)
            flatpak-packages {
                com.spotify.Client
                org.telegram.desktop
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 3);
        assert_eq!(config.soar_packages.len(), 4);
        assert_eq!(config.flatpak_packages.len(), 2);
    }

    // New syntax tests

    #[test]
    fn test_parse_colon_syntax_soar() {
        let kdl = r#"
            packages:soar {
                bat
                exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));
    }

    #[test]
    fn test_parse_colon_syntax_aur() {
        let kdl = r#"
            packages:aur {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));
    }

    #[test]
    fn test_parse_colon_syntax_flatpak() {
        let kdl = r#"
            packages:flatpak {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
        assert!(config.flatpak_packages.contains(&"org.mozilla.firefox".to_string()));
    }

    #[test]
    fn test_parse_embedded_syntax() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                    org.mozilla.firefox
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));

        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
        assert!(config.flatpak_packages.contains(&"org.mozilla.firefox".to_string()));
    }

    #[test]
    fn test_parse_mixed_syntax_styles() {
        let kdl = r#"
            // Default packages (AUR)
            packages {
                hyprland
                waybar
            }

            // Colon syntax for Soar
            packages:soar {
                bat
            }

            // Colon syntax for Flatpak
            packages:flatpak {
                com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.soar_packages.len(), 1);
        assert_eq!(config.flatpak_packages.len(), 1);
    }

    #[test]
    fn test_parse_embedded_with_explicit_soar() {
        let kdl = r#"
            packages {
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
    }
}
