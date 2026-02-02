//! Backend parser registry
//!
//! Manages all available backend parsers and provides
//! a unified interface for parsing packages from KDL nodes.

use crate::config::kdl_modules::parsers::BackendParser;
use crate::config::kdl_modules::types::{Backend, PackageEntry, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

// Import individual parsers
use crate::config::kdl_modules::parsers::{
    AurParser, BrewParser, BunParser, CargoParser, FlatpakParser, NpmParser, PipParser, PnpmParser,
    SoarParser, YarnParser,
};

/// Registry for backend parsers
///
/// This registry manages all available backend parsers and provides
/// a unified interface for parsing packages from KDL nodes.
pub struct BackendParserRegistry {
    parsers: Vec<Box<dyn BackendParser>>,
    #[allow(dead_code)]
    default_backend: &'static str, // Reserved for future use
}

impl BackendParserRegistry {
    /// Create a new registry with default parsers
    pub fn new() -> Self {
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
    pub fn find_parser(&self, backend: &str) -> Option<&dyn BackendParser> {
        self.parsers
            .iter()
            .find(|p| p.matches(backend))
            .map(|p| p.as_ref())
    }

    /// Parse packages with inline prefix syntax
    ///
    /// Handles syntax like: `packages { aur:hyprland soar:bat }`
    pub fn parse_inline_prefix(&self, package_str: &str, config: &mut RawConfig) -> Result<()> {
        if let Some((backend, package)) = package_str.split_once(':') {
            if self.find_parser(backend).is_some() {
                let entry = PackageEntry {
                    name: package.to_string(),
                };

                // Directly add to the appropriate config vector based on backend
                match backend {
                    "aur" => config.packages_for_mut(&Backend::Aur).push(entry),
                    "soar" | "app" => config.packages_for_mut(&Backend::Soar).push(entry),
                    "flatpak" => config.packages_for_mut(&Backend::Flatpak).push(entry),
                    "npm" => config.packages_for_mut(&Backend::Npm).push(entry),
                    "yarn" => config.packages_for_mut(&Backend::Yarn).push(entry),
                    "pnpm" => config.packages_for_mut(&Backend::Pnpm).push(entry),
                    "bun" => config.packages_for_mut(&Backend::Bun).push(entry),
                    "pip" => config.packages_for_mut(&Backend::Pip).push(entry),
                    "cargo" => config.packages_for_mut(&Backend::Cargo).push(entry),
                    "brew" => config.packages_for_mut(&Backend::Brew).push(entry),
                    _ => config.packages_for_mut(&Backend::Aur).push(entry),
                }
            } else {
                // Unknown backend - treat the whole string as package name with default backend
                config.packages_for_mut(&Backend::Aur).push(PackageEntry {
                    name: package_str.to_string(),
                });
            }
        } else {
            // No prefix - use default backend (AUR)
            config.packages_for_mut(&Backend::Aur).push(PackageEntry {
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
    pub fn parse_packages_node(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
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
        if let Some(_children) = node.children() {
            for child in _children.nodes() {
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
                        config.packages_for_mut(&Backend::Aur).push(PackageEntry {
                            name: child_name.to_string(),
                        });
                    }

                    // Also check for string arguments in the child node
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            if val.contains(':') {
                                self.parse_inline_prefix(val, config)?;
                            } else {
                                config.packages_for_mut(&Backend::Aur).push(PackageEntry {
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
                    config.packages_for_mut(&Backend::Aur).push(PackageEntry {
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
