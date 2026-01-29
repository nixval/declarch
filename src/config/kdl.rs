// Re-export types from kdl_modules for backward compatibility
pub use crate::config::kdl_modules::types::{
    ConflictEntry, ConfigMeta, ErrorBehavior, HookCondition, HookConfig, HookEntry,
    HookPhase, HookType, PackageEntry, PolicyConfig, RawConfig,
};
// Re-export BackendParser trait from parsers module
pub use crate::config::kdl_modules::parsers::BackendParser;
// Re-export helper functions for backward compatibility
pub use crate::config::kdl_modules::helpers::{
    aliases, conflicts, env, hooks, meta, packages, policy, repositories,
};

use crate::error::Result;
use kdl::{KdlDocument, KdlNode};
use std::collections::HashMap;

// Import individual parsers
use crate::config::kdl_modules::parsers::{
    AurParser, BrewParser, BunParser, CargoParser, FlatpakParser, NpmParser,
    PipParser, PnpmParser, SoarParser, YarnParser,
};

/// Registry for backend parsers
///
/// This registry manages all available backend parsers and provides
/// a unified interface for parsing packages from KDL nodes.
struct BackendParserRegistry {
    parsers: Vec<Box<dyn BackendParser>>,
    #[allow(dead_code)]
    default_backend: &'static str, // Reserved for future use
}

impl BackendParserRegistry {
    /// Create a new registry with default parsers
    fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(AurParser),
                Box::new(SoarParser),
                Box::new(FlatpakParser),
                Box::new(NpmParser),
                Box::new(YarnParser),
                Box::new(PnpmParser),
                Box::new(BunParser),
                Box::new(PipParser),
                Box::new(CargoParser),
                Box::new(BrewParser),
            ],
            default_backend: "aur", // Default to AUR for Arch Linux
        }
    }

    /// Find a parser by backend name (including aliases)
    fn find_parser(&self, backend: &str) -> Option<&dyn BackendParser> {
        self.parsers
            .iter()
            .find(|p| p.matches(backend))
            .map(|p| p.as_ref())
    }

    /// Parse packages with inline prefix syntax
    ///
    /// Handles syntax like: `packages { aur:hyprland soar:bat }`
    fn parse_inline_prefix(&self, package_str: &str, config: &mut RawConfig) -> Result<()> {
        if let Some((backend, package)) = package_str.split_once(':') {
            if self.find_parser(backend).is_some() {
                let entry = PackageEntry {
                    name: package.to_string(),
                };

                // Directly add to the appropriate config vector based on backend
                match backend {
                    "aur" => config.packages.push(entry),
                    "soar" | "app" => config.soar_packages.push(entry),
                    "flatpak" => config.flatpak_packages.push(entry),
                    "npm" => config.npm_packages.push(entry),
                    "yarn" => config.yarn_packages.push(entry),
                    "pnpm" => config.pnpm_packages.push(entry),
                    "bun" => config.bun_packages.push(entry),
                    "pip" => config.pip_packages.push(entry),
                    "cargo" => config.cargo_packages.push(entry),
                    "brew" => config.brew_packages.push(entry),
                    _ => config.packages.push(entry),
                }
            } else {
                // Unknown backend - treat the whole string as package name with default backend
                config.packages.push(PackageEntry {
                    name: package_str.to_string(),
                });
            }
        } else {
            // No prefix - use default backend (AUR)
            config.packages.push(PackageEntry {
                name: package_str.to_string(),
            });
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
            // Custom backend - parse and store in custom_packages HashMap
            return self.parse_custom_backend(backend, node, config);
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
                        config.packages.push(PackageEntry {
                            name: child_name.to_string(),
                        });
                    }

                    // Also check for string arguments in the child node
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            if val.contains(':') {
                                self.parse_inline_prefix(val, config)?;
                            } else {
                                config.packages.push(PackageEntry {
                                    name: val.to_string(),
                                });
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
                    config.packages.push(PackageEntry {
                        name: val.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Parse packages for a custom (user-defined) backend
    fn parse_custom_backend(
        &self,
        backend_name: &str,
        node: &KdlNode,
        config: &mut RawConfig,
    ) -> Result<()> {
        let mut packages = Vec::new();

        // Extract packages from children
        if let Some(children) = node.children() {
            for child in children.nodes() {
                let child_name = child.name().value();
                packages.push(PackageEntry {
                    name: child_name.to_string(),
                });

                // Also check for string arguments
                for entry in child.entries() {
                    if let Some(val) = entry.value().as_string() {
                        packages.push(PackageEntry {
                            name: val.to_string(),
                        });
                    }
                }
            }
        }

        // Extract packages from direct arguments
        for entry in node.entries() {
            if let Some(val) = entry.value().as_string() {
                packages.push(PackageEntry {
                    name: val.to_string(),
                });
            }
        }

        // Store in custom_packages HashMap
        config
            .custom_packages
            .insert(backend_name.to_string(), packages);

        Ok(())
    }
}

impl Default for BackendParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse backend options: options:aur { noconfirm true }
fn parse_backend_options(
    node: &KdlNode,
    options: &mut HashMap<String, HashMap<String, String>>,
) -> Result<()> {
    // Check for colon syntax: options:aur
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified, apply to all? Or skip?
        // For now, skip if no backend specified
        return Ok(());
    };

    let mut opts = HashMap::new();

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            if let Some(val) = meta::get_first_string(child) {
                opts.insert(key.to_string(), val);
            } else if let Some(val) = child.entries().first()
                && let Some(val) = val.value().as_string()
            {
                opts.insert(key.to_string(), val.to_string());
            } else {
                // Boolean flag without value
                opts.insert(key.to_string(), "true".to_string());
            }
        }
    }

    // Extract from string arguments (key=value format)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string()
            && let Some((key, v)) = val.split_once('=')
        {
            opts.insert(key.to_string(), v.to_string());
        }
    }

    if !opts.is_empty() {
        options.insert(backend_name, opts);
    }

    Ok(())
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
        npm_packages: vec![],
        yarn_packages: vec![],
        pnpm_packages: vec![],
        bun_packages: vec![],
        pip_packages: vec![],
        cargo_packages: vec![],
        brew_packages: vec![],
        custom_packages: HashMap::new(),
        excludes: vec![],
        aliases: HashMap::new(),
        editor: None,
        meta: ConfigMeta::default(),
        conflicts: vec![],
        backend_options: HashMap::new(),
        env: HashMap::new(),
        repositories: HashMap::new(),
        policy: PolicyConfig::default(),
        hooks: HookConfig::default(),
    };

    let registry = BackendParserRegistry::new();

    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            "import" | "imports" => {
                packages::extract_strings(node, &mut config.imports);
            }
            "exclude" | "excludes" => {
                packages::extract_mixed_values(node, &mut config.excludes);
            }
            "aliases-pkg" | "alias-pkg" => {
                aliases::extract_aliases(node, &mut config.aliases);
            }
            "editor" => {
                // Extract editor from first string argument
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.editor = Some(val.to_string());
                }
            }
            "description" => {
                // Parse description into meta
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.meta.description = Some(val.to_string());
                }
            }
            // NEW: Meta block
            "meta" => {
                meta::parse_meta_block(node, &mut config.meta)?;
            }
            // NEW: Conflicts
            "conflicts" | "conflict" => {
                conflicts::parse_conflicts(node, &mut config.conflicts)?;
            }
            // NEW: Backend options
            name if name.starts_with("options") => {
                parse_backend_options(node, &mut config.backend_options)?;
            }
            // NEW: Environment variables
            name if name.starts_with("env") => {
                env::parse_env_vars(node, &mut config.env, None)?;
            }
            // NEW: Package repositories
            name if name.starts_with("repos") || name.starts_with("repositories") => {
                repositories::parse_repositories(node, &mut config.repositories)?;
            }
            // NEW: Policy
            "policy" => {
                policy::parse_policy(node, &mut config.policy)?;
            }
            // NEW: Hooks
            "hooks" => {
                hooks::parse_hooks(node, &mut config.hooks)?;
            }
            // NEW: Simplified flat hooks (backward compatibility)
            "on-sync" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.hooks.hooks.push(HookEntry {
                        command: val,
                        hook_type: HookType::User,
                        phase: HookPhase::PostSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            "on-sync-sudo" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.hooks.hooks.push(HookEntry {
                        command: val,
                        hook_type: HookType::Root,
                        phase: HookPhase::PostSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            "on-pre-sync" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.hooks.hooks.push(HookEntry {
                        command: val,
                        hook_type: HookType::User,
                        phase: HookPhase::PreSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            // Parse packages with flexible syntax using the registry
            name if name.starts_with("packages") => {
                registry.parse_packages_node(node, &mut config)?;
            }
            // Legacy syntax support (with deprecation warning in the future)
            "aur-packages" | "aur-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "soar-packages" | "soar-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .soar_packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "flatpak-packages" | "flatpak-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .flatpak_packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            _ => {}
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests (unchanged for backward compatibility)

    #[test]
    fn aliases_inline() {
        let kdl = r#"
            aliases-pkg pipewire pipewire-jack2
            aliases-pkg python-poetry python-poetry-core
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.aliases.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn aliases_block() {
        let kdl = r#"
            aliases-pkg {
                pipewire pipewire-jack2
                python-poetry python-poetry-core
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.aliases.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn mixed_config() {
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
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
    }

    #[test]
    fn empty_aliases() {
        let kdl = r#"
            packages neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn default_packages() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                swww
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
        assert!(config.packages.iter().any(|p| p.name == "swww"));
    }

    #[test]
    fn cross_distro() {
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
    fn soar_colon() {
        let kdl = r#"
            packages:soar {
                bat
                exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn aur_colon() {
        let kdl = r#"
            packages:aur {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
    }

    #[test]
    fn flatpak_colon() {
        let kdl = r#"
            packages:flatpak {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn nested_blocks() {
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
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn mixed_formats() {
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
    fn soar_nested() {
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
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    // NEW TESTS: Inline prefix syntax

    #[test]
    fn inline_single() {
        let kdl = r#"
            packages {
                soar:bat
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
    }

    #[test]
    fn inline_multiple() {
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
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn inline_nested() {
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
        assert_eq!(config.packages.len(), 2); // hyprland + waybar
        assert_eq!(config.soar_packages.len(), 1); // bat
        assert_eq!(config.flatpak_packages.len(), 1); // com.spotify.Client
    }

    #[test]
    fn inline_alias() {
        let kdl = r#"
            packages {
                app:bat
                app:exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn string_args() {
        let kdl = r#"
            packages "soar:bat" "aur:hyprland" "flatpak:app.id"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));

        assert_eq!(config.packages.len(), 1);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.iter().any(|p| p.name == "app.id"));
    }

    #[test]
    fn unknown_pm() {
        let kdl = r#"
            packages {
                unknown:package
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Unknown backend should be treated as package name with default backend
        assert!(config.packages.iter().any(|p| p.name == "unknown:package"));
    }

    #[test]
    fn complex_config() {
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
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
        assert!(config.packages.iter().any(|p| p.name == "swww"));
        assert!(config.packages.iter().any(|p| p.name == "rofi"));

        // Soar packages: bat, exa
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        // Flatpak packages: com.spotify.Client
        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn registry() {
        let registry = BackendParserRegistry::new();

        // Test finding parsers by name
        assert!(registry.find_parser("aur").is_some());
        assert!(registry.find_parser("soar").is_some());
        assert!(registry.find_parser("flatpak").is_some());
        assert!(registry.find_parser("unknown").is_none());

        // Test aliases
        assert!(registry.find_parser("app").is_some()); // alias for soar
    }

    #[test]
    fn backward_compat() {
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
        assert_eq!(config.soar_packages.len(), 2); // bat + exa
        assert_eq!(config.flatpak_packages.len(), 2);
    }

    // NEW: Meta block tests

    #[test]
    fn meta_block() {
        let kdl = r#"
            meta {
                description "My Hyprland Setup"
                author "nixval"
                version "1.0.0"
                url "https://github.com/nixval/dotfiles"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(
            config.meta.description,
            Some("My Hyprland Setup".to_string())
        );
        assert_eq!(config.meta.author, Some("nixval".to_string()));
        assert_eq!(config.meta.version, Some("1.0.0".to_string()));
        assert_eq!(
            config.meta.url,
            Some("https://github.com/nixval/dotfiles".to_string())
        );
    }

    #[test]
    fn meta_tags() {
        let kdl = r#"
            meta {
                description "Workstation setup"
                tags "workstation" "hyprland" "development"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.meta.tags.len(), 3);
        assert!(config.meta.tags.contains(&"workstation".to_string()));
        assert!(config.meta.tags.contains(&"hyprland".to_string()));
        assert!(config.meta.tags.contains(&"development".to_string()));
    }

    // NEW: Conflicts tests

    #[test]
    fn conflicts() {
        let kdl = r#"
            conflicts vim neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 2);
        assert!(config.conflicts[0].packages.contains(&"vim".to_string()));
        assert!(config.conflicts[0].packages.contains(&"neovim".to_string()));
    }

    // NEW: Backend options tests

    #[test]
    fn aur_options() {
        let kdl = r#"
            options:aur {
                noconfirm
                helper "paru"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.backend_options.contains_key("aur"));
        let aur_opts = &config.backend_options["aur"];
        assert_eq!(aur_opts.get("noconfirm"), Some(&"true".to_string()));
        assert_eq!(aur_opts.get("helper"), Some(&"paru".to_string()));
    }

    // NEW: Environment variables tests

    #[test]
    fn env_vars() {
        let kdl = r#"
            env EDITOR="nvim" VISUAL="nvim"

            env:aur MAKEFLAGS="-j4"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.env.contains_key("global"));
        assert!(config.env.contains_key("aur"));

        let global_env = &config.env["global"];
        assert!(global_env.contains(&"EDITOR=nvim".to_string()));
        assert!(global_env.contains(&"VISUAL=nvim".to_string()));

        let aur_env = &config.env["aur"];
        assert!(aur_env.contains(&"MAKEFLAGS=-j4".to_string()));
    }

    // NEW: Repositories tests

    #[test]
    fn repositories() {
        let kdl = r#"
            repos:aur {
                "https://aur.archlinux.org"
            }

            repos:flatpak {
                "https://flathub.org/repo/flathub.flatpakrepo"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.repositories.contains_key("aur"));
        assert!(config.repositories.contains_key("flatpak"));

        assert!(config.repositories["aur"].contains(&"https://aur.archlinux.org".to_string()));
        assert!(
            config.repositories["flatpak"]
                .contains(&"https://flathub.org/repo/flathub.flatpakrepo".to_string())
        );
    }

    // NEW: Policy tests

    #[test]
    fn policy() {
        let kdl = r#"
            policy {
                protected {
                    linux
                    systemd
                    base-devel
                }
                orphans "ask"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.policy.protected.contains("linux"));
        assert!(config.policy.protected.contains("systemd"));
        assert!(config.policy.protected.contains("base-devel"));
        assert_eq!(config.policy.orphans, Some("ask".to_string()));
    }

    // NEW: Hooks tests

    #[test]
    fn hooks() {
        let kdl = r#"
            hooks {
                post-sync "notify-send 'Packages updated'"
                post-sync "systemctl restart gdm" --sudo
                post-sync "~/.config/declarch/post-sync.sh"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Filter post-sync hooks
        let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .collect();

        assert_eq!(post_sync_hooks.len(), 3);

        assert_eq!(
            post_sync_hooks[0].command,
            "notify-send 'Packages updated'"
        );
        assert_eq!(post_sync_hooks[0].hook_type, HookType::User);

        assert_eq!(post_sync_hooks[1].command, "systemctl restart gdm");
        assert_eq!(post_sync_hooks[1].hook_type, HookType::Root);

        assert_eq!(
            post_sync_hooks[2].command,
            "~/.config/declarch/post-sync.sh"
        );
        assert_eq!(post_sync_hooks[2].hook_type, HookType::User);
    }

    // NEW: Comprehensive integration test

    #[test]
    fn full_config() {
        let kdl = r#"
            meta {
                description "Full workstation setup"
                author "nixval"
                version "2.0.0"
            }

            packages {
                hyprland
                neovim
                waybar
            }

            packages:soar {
                bat
                exa
            }

            conflicts {
                vim neovim
                pipewire pulseaudio
            }

            options:aur {
                noconfirm
            }

            env EDITOR="nvim"

            policy {
                protected {
                    linux
                    systemd
                }
                orphans "keep"
            }

            hooks {
                post-sync "notify-send 'Sync complete'"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check meta
        assert_eq!(
            config.meta.description,
            Some("Full workstation setup".to_string())
        );
        assert_eq!(config.meta.author, Some("nixval".to_string()));
        assert_eq!(config.meta.version, Some("2.0.0".to_string()));

        // Check packages
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "neovim"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        // Check conflicts (1 conflict entry with 4 packages all mutually exclusive)
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 4);

        // Check options
        assert!(config.backend_options.contains_key("aur"));

        // Check env
        assert!(config.env.contains_key("global"));

        // Check policy
        assert!(config.policy.protected.contains("linux"));
        assert_eq!(config.policy.orphans, Some("keep".to_string()));

        // Check hooks
        let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 1);
    }

    // NEW: Flat hooks syntax test

    #[test]
    fn hooks_flat() {
        let kdl = r#"
            on-sync "notify-send 'Packages updated'"
            on-sync-sudo "systemctl restart gdm"
            on-pre-sync "echo 'Starting sync...'"
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check pre-sync hooks
        let pre_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PreSync)
            .collect();
        assert_eq!(pre_sync_hooks.len(), 1);
        assert_eq!(pre_sync_hooks[0].command, "echo 'Starting sync...'");
        assert_eq!(pre_sync_hooks[0].hook_type, HookType::User);

        // Check post-sync hooks
        let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 2);
        assert_eq!(
            post_sync_hooks[0].command,
            "notify-send 'Packages updated'"
        );
        assert_eq!(post_sync_hooks[0].hook_type, HookType::User);
        assert_eq!(post_sync_hooks[1].command, "systemctl restart gdm");
        assert_eq!(post_sync_hooks[1].hook_type, HookType::Root);
    }

    // NEW: Mixed hooks (flat syntax + hooks block)

    #[test]
    fn hooks_mixed() {
        let kdl = r#"
            on-sync "notify-send 'Flat hook'"

            hooks {
                post-sync "notify-send 'Nested hook'"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Filter post-sync hooks
        let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .collect();

        // Should have both flat and nested hooks
        assert_eq!(post_sync_hooks.len(), 2);
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "notify-send 'Flat hook'")
        );
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "notify-send 'Nested hook'")
        );
    }

    // Comprehensive hooks tests

    #[test]
    fn hooks_comprehensive() {
        let kdl = r#"
meta {
    description "Test all hooks features"
    author "nixval"
    version "1.0.0"
}

packages {
    bat
}

hooks {
    // Global hooks
    pre-sync "echo 'Pre-sync'"
    post-sync "echo 'Post-sync'"
    on-success "echo 'Success'"
    on-failure "echo 'Failed'"

    // Package hook (block syntax)
    bat {
        post-install "echo 'Bat installed'"
    }

    // Backend hooks
    aur:post-sync "echo 'AUR synced'"
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        // Test pre-sync hooks
        let pre_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PreSync)
            .collect();
        assert_eq!(pre_sync_hooks.len(), 1);
        assert_eq!(pre_sync_hooks[0].command, "echo 'Pre-sync'");
        assert!(pre_sync_hooks[0].package.is_none());

        // Test post-sync hooks
        let post_sync_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 2);
        assert!(post_sync_hooks.iter().any(|h| h.command == "echo 'Post-sync'"));
        assert!(post_sync_hooks.iter().any(|h| h.command == "echo 'AUR synced'"));

        // Test on-success hooks
        let on_success_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::OnSuccess)
            .collect();
        assert_eq!(on_success_hooks.len(), 1);
        assert_eq!(on_success_hooks[0].command, "echo 'Success'");

        // Test on-failure hooks
        let on_failure_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::OnFailure)
            .collect();
        assert_eq!(on_failure_hooks.len(), 1);
        assert_eq!(on_failure_hooks[0].command, "echo 'Failed'");

        // Test post-install hooks
        let post_install_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostInstall)
            .collect();
        assert_eq!(post_install_hooks.len(), 1);
        assert_eq!(post_install_hooks[0].command, "echo 'Bat installed'");
        assert_eq!(post_install_hooks[0].package.as_ref().unwrap(), "bat");
    }

    #[test]
    fn hooks_flags() {
        let kdl = r#"
hooks {
    post-sync "echo 'Regular hook'"
    post-sync "systemctl restart gdm" --sudo
    docker:post-install "mkinitcpio -P" --sudo --required
    post-sync "cleanup.sh" --ignore
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        let all_hooks = &config.hooks.hooks;

        // Regular hook (no sudo)
        let regular = all_hooks.iter()
            .find(|h| h.command.contains("Regular hook"))
            .unwrap();
        assert_eq!(regular.hook_type, HookType::User);
        assert_eq!(regular.error_behavior, ErrorBehavior::Warn);

        // Sudo hook
        let sudo = all_hooks.iter()
            .find(|h| h.command.contains("systemctl restart gdm"))
            .unwrap();
        assert_eq!(sudo.hook_type, HookType::Root);

        // Required hook
        let required = all_hooks.iter()
            .find(|h| h.command.contains("mkinitcpio"))
            .unwrap();
        assert_eq!(required.hook_type, HookType::Root);
        assert_eq!(required.error_behavior, ErrorBehavior::Required);

        // Ignore hook
        let ignore = all_hooks.iter()
            .find(|h| h.command.contains("cleanup.sh"))
            .unwrap();
        assert_eq!(ignore.error_behavior, ErrorBehavior::Ignore);
    }

    #[test]
    fn hooks_shorthand() {
        let kdl = r#"
packages {
    docker
    waybar
}

hooks {
    // Shorthand syntax
    docker:post-install "systemctl enable docker" --sudo
    waybar:post-install "pkill waybar"

    // Backend shorthand
    aur:post-sync "echo 'AUR done'"
    flatpak:post-sync "echo 'Flatpak done'"
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        // Test package shorthand
        let docker_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.package.as_deref() == Some("docker"))
            .collect();
        assert_eq!(docker_hooks.len(), 1);
        assert_eq!(docker_hooks[0].phase, HookPhase::PostInstall);
        assert_eq!(docker_hooks[0].hook_type, HookType::Root);

        let waybar_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.package.as_deref() == Some("waybar"))
            .collect();
        assert_eq!(waybar_hooks.len(), 1);
        assert_eq!(waybar_hooks[0].phase, HookPhase::PostInstall);

        // Test backend shorthand (should be detected by post-sync phase)
        let aur_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .filter(|h| h.command.contains("AUR"))
            .collect();
        assert_eq!(aur_hooks.len(), 1);

        let flatpak_hooks: Vec<_> = config.hooks.hooks.iter()
            .filter(|h| h.phase == HookPhase::PostSync)
            .filter(|h| h.command.contains("Flatpak"))
            .collect();
        assert_eq!(flatpak_hooks.len(), 1);
    }
}
