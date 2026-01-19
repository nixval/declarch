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
    /// Syntax: packages:soar { ... } or soar:package in packages block
    pub soar_packages: Vec<String>,
    /// Flatpak packages
    /// Syntax: packages:flatpak { ... } or flatpak:package in packages block
    pub flatpak_packages: Vec<String>,
    pub excludes: Vec<String>,
    /// Package aliases: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub aliases: HashMap<String, String>,
    /// Editor to use for edit command
    /// Syntax: editor "nvim" or editor nvim
    pub editor: Option<String>,
}

/// Trait for backend-specific package parsing
///
/// Each backend (AUR, Soar, Flatpak) implements this trait
/// to define how it parses packages from KDL nodes.
pub trait BackendParser: Send + Sync {
    /// Backend identifier (e.g., "aur", "soar", "flatpak")
    fn name(&self) -> &'static str;

    /// Aliases for this backend (e.g., "app" is an alias for "soar")
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// Parse packages from a KDL node and add them to the config
    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()>;

    /// Check if a backend name matches this parser (including aliases)
    fn matches(&self, backend: &str) -> bool {
        self.name() == backend || self.aliases().contains(&backend)
    }
}

/// AUR (Arch User Repository) backend parser
struct AurParser;

impl BackendParser for AurParser {
    fn name(&self) -> &'static str {
        "aur"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.packages);
        Ok(())
    }
}

/// Soar (static binaries) backend parser
struct SoarParser;

impl BackendParser for SoarParser {
    fn name(&self) -> &'static str {
        "soar"
    }

    fn aliases(&self) -> &[&'static str] {
        &["app"]  // "app" is an alias for "soar"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.soar_packages);
        Ok(())
    }
}

/// Flatpak backend parser
struct FlatpakParser;

impl BackendParser for FlatpakParser {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.flatpak_packages);
        Ok(())
    }
}

/// Registry for backend parsers
///
/// This registry manages all available backend parsers and provides
/// a unified interface for parsing packages from KDL nodes.
struct BackendParserRegistry {
    parsers: Vec<Box<dyn BackendParser>>,
    #[allow(dead_code)]
    default_backend: &'static str,  // Reserved for future use
}

impl BackendParserRegistry {
    /// Create a new registry with default parsers
    fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(AurParser),
                Box::new(SoarParser),
                Box::new(FlatpakParser),
            ],
            default_backend: "aur",  // Default to AUR for Arch Linux
        }
    }

    /// Find a parser by backend name (including aliases)
    fn find_parser(&self, backend: &str) -> Option<&dyn BackendParser> {
        self.parsers.iter().find(|p| p.matches(backend)).map(|p| p.as_ref())
    }

    /// Parse packages with inline prefix syntax
    ///
    /// Handles syntax like: `packages { aur:hyprland soar:bat }`
    fn parse_inline_prefix(&self, package_str: &str, config: &mut RawConfig) -> Result<()> {
        if let Some((backend, package)) = package_str.split_once(':') {
            if self.find_parser(backend).is_some() {
                // Directly add to the appropriate config vector based on backend
                match backend {
                    "aur" => config.packages.push(package.to_string()),
                    "soar" | "app" => config.soar_packages.push(package.to_string()),
                    "flatpak" => config.flatpak_packages.push(package.to_string()),
                    _ => config.packages.push(package.to_string()),
                }
            } else {
                // Unknown backend - treat the whole string as package name with default backend
                config.packages.push(package_str.to_string());
            }
        } else {
            // No prefix - use default backend (AUR)
            extract_package_to(package_str, &mut config.packages);
        }
        Ok(())
    }

    /// Parse a packages node with flexible syntax
    ///
    /// Supported syntaxes:
    /// 1. `packages { hyprland waybar }` → AUR packages (default)
    /// 2. `packages:aur { hyprland }` → AUR packages (explicit)
    /// 3. `packages:soar { bat exa }` → Soar packages
    /// 4. `packages:flatpak { com.spotify.Client }` → Flatpak packages
    /// 5. `packages { bat aur:hyprland flatpak:app.id }` → Mixed with inline prefix
    /// 6. `packages { soar { bat } flatpak { app.id } }` → Nested blocks
    fn parse_packages_node(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        let node_name = node.name().value();

        // Case 1: Colon syntax: packages:backend
        if let Some((_, backend)) = node_name.split_once(':') {
            if let Some(parser) = self.find_parser(backend) {
                return parser.parse(node, config);
            }
            // Unknown backend - fall through to default parsing
        }

        // Case 2: Check for nested children (backend blocks)
        if let Some(children) = node.children() {
            for child in children.nodes() {
                let child_name = child.name().value();

                // Check if child name is a backend identifier
                if let Some(parser) = self.find_parser(child_name) {
                    // Parse as backend block: `packages { aur { ... } }`
                    parser.parse(child, config)?;
                } else {
                    // Check for inline prefix syntax: `aur:hyprland`
                    if child_name.contains(':') {
                        self.parse_inline_prefix(child_name, config)?;
                    } else {
                        // No backend prefix - use default backend
                        extract_package_to(child_name, &mut config.packages);
                    }

                    // Also check for string arguments in the child node
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            if val.contains(':') {
                                self.parse_inline_prefix(val, config)?;
                            } else {
                                extract_package_to(val, &mut config.packages);
                            }
                        }
                    }
                }
            }
        }

        // Case 3: Extract direct string arguments (default to AUR)
        for entry in node.entries() {
            if let Some(val) = entry.value().as_string() {
                if val.contains(':') {
                    self.parse_inline_prefix(val, config)?;
                } else {
                    extract_package_to(val, &mut config.packages);
                }
            }
        }

        Ok(())
    }
}

impl Default for BackendParserRegistry {
    fn default() -> Self {
        Self::new()
    }
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

    let registry = BackendParserRegistry::new();

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
            // Parse packages with flexible syntax using the registry
            name if name.starts_with("packages") => {
                registry.parse_packages_node(node, &mut config)?;
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

/// Extract packages from a node and add them to a target vector
///
/// Handles:
/// - String arguments: `packages "bat" "exa"`
/// - Children node names: `packages { bat exa }`
/// - Mixed: `packages "bat" { exa }`
fn extract_packages_to(node: &KdlNode, target: &mut Vec<String>) {
    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    // Extract from children node names
    if let Some(children) = node.children() {
        for child in children.nodes() {
            target.push(child.name().value().to_string());

            // Also check for string arguments in child nodes
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

/// Extract a single package string and add it to a target vector
fn extract_package_to(package: &str, target: &mut Vec<String>) {
    target.push(package.to_string());
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

    // Existing tests (unchanged for backward compatibility)

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

    // NEW TESTS: Inline prefix syntax

    #[test]
    fn test_parse_inline_prefix_single() {
        let kdl = r#"
            packages {
                soar:bat
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.contains(&"bat".to_string()));
    }

    #[test]
    fn test_parse_inline_prefix_multiple() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar:bat
                soar:exa
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Default (hyprland) + aur:waybar
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
    }

    #[test]
    fn test_parse_inline_prefix_with_nested_blocks() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar {
                    bat
                }
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);  // hyprland + waybar
        assert_eq!(config.soar_packages.len(), 1);  // bat
        assert_eq!(config.flatpak_packages.len(), 1);  // com.spotify.Client
    }

    #[test]
    fn test_parse_inline_prefix_with_app_alias() {
        let kdl = r#"
            packages {
                app:bat
                app:exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));
    }

    #[test]
    fn test_parse_inline_prefix_string_arguments() {
        let kdl = r#"
            packages "soar:bat" "aur:hyprland" "flatpak:app.id"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.contains(&"bat".to_string()));

        assert_eq!(config.packages.len(), 1);
        assert!(config.packages.contains(&"hyprland".to_string()));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.contains(&"app.id".to_string()));
    }

    #[test]
    fn test_parse_unknown_backend_with_inline_prefix() {
        let kdl = r#"
            packages {
                unknown:package
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Unknown backend should be treated as package name with default backend
        assert!(config.packages.contains(&"unknown:package".to_string()));
    }

    #[test]
    fn test_parse_complex_mixed_syntax() {
        let kdl = r#"
            packages {
                // Default packages (AUR)
                hyprland
                waybar

                // Inline prefix syntax
                soar:bat
                flatpak:com.spotify.Client

                // Nested blocks
                aur {
                    swww
                }

                // Mixed inline and nested
                soar {
                    exa
                }
                aur:rofi
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // AUR packages: hyprland, waybar, swww, rofi
        assert_eq!(config.packages.len(), 4);
        assert!(config.packages.contains(&"hyprland".to_string()));
        assert!(config.packages.contains(&"waybar".to_string()));
        assert!(config.packages.contains(&"swww".to_string()));
        assert!(config.packages.contains(&"rofi".to_string()));

        // Soar packages: bat, exa
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.contains(&"bat".to_string()));
        assert!(config.soar_packages.contains(&"exa".to_string()));

        // Flatpak packages: com.spotify.Client
        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.contains(&"com.spotify.Client".to_string()));
    }

    #[test]
    fn test_backend_parser_registry() {
        let registry = BackendParserRegistry::new();

        // Test finding parsers by name
        assert!(registry.find_parser("aur").is_some());
        assert!(registry.find_parser("soar").is_some());
        assert!(registry.find_parser("flatpak").is_some());
        assert!(registry.find_parser("unknown").is_none());

        // Test aliases
        assert!(registry.find_parser("app").is_some());  // alias for soar
    }

    #[test]
    fn test_backward_compatibility() {
        // Ensure all old syntax still works
        let kdl = r#"
            packages {
                hyprland
                waybar
            }

            packages:soar {
                bat
            }

            packages:flatpak {
                com.spotify.Client
            }

            soar-packages {
                exa
            }

            flatpak-packages {
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.soar_packages.len(), 2);  // bat + exa
        assert_eq!(config.flatpak_packages.len(), 2);
    }
}
