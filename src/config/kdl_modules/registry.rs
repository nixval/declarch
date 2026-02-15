//! Backend parser registry (DEPRECATED in v0.6+)
//!
//! This file is kept for backward compatibility but is NO LONGER USED.
//!
//! In v0.6+, all parsing is handled by the generic parser in parser.rs.
//! There are no backend-specific parsers.
//!
//! This module will be removed in v0.7.

use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// DEPRECATED: Backend parser trait
///
/// This trait is no longer used. All parsing is now generic.
pub trait BackendParser: Send + Sync {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &[&'static str] {
        &[]
    }
    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()>;
    fn matches(&self, backend: &str) -> bool {
        self.name() == backend || self.aliases().contains(&backend)
    }
}

/// DEPRECATED: Backend parser registry
///
/// This registry no longer manages individual parsers.
/// It exists only for backward compatibility during the transition.
pub struct BackendParserRegistry;

impl BackendParserRegistry {
    /// Create a new registry (empty, no parsers needed in v0.6+)
    pub fn new() -> Self {
        Self
    }

    /// DEPRECATED: Parse packages node.
    ///
    /// Compatibility adapter that delegates to the unified package map format.
    pub fn parse_packages_node(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        let node_name = node.name().value();

        // Support pkg:backend { ... } shorthand
        if let Some((prefix, backend)) = node_name.split_once(':')
            && prefix == "pkg"
        {
            let mut packages = Vec::new();
            crate::config::kdl_modules::helpers::packages::extract_packages_to(node, &mut packages);
            if !packages.is_empty() {
                config
                    .packages_by_backend
                    .entry(backend.to_string())
                    .or_default()
                    .extend(packages);
            }
            return Ok(());
        }

        // Support pkg { backend { ... } backend:pkg ... }
        if node_name == "pkg"
            && let Some(children) = node.children()
        {
            for child in children.nodes() {
                let child_name = child.name().value();
                if let Some((backend, package)) = child_name.split_once(':') {
                    config
                        .packages_by_backend
                        .entry(backend.to_string())
                        .or_default()
                        .push(crate::config::kdl_modules::types::PackageEntry {
                            name: package.to_string(),
                        });
                } else {
                    let mut packages = Vec::new();
                    crate::config::kdl_modules::helpers::packages::extract_packages_to(
                        child,
                        &mut packages,
                    );
                    if !packages.is_empty() {
                        config
                            .packages_by_backend
                            .entry(child_name.to_string())
                            .or_default()
                            .extend(packages);
                    }
                }
            }
        }

        Ok(())
    }

    /// DEPRECATED: Parse inline prefix
    pub fn parse_inline_prefix(&self, package_str: &str, config: &mut RawConfig) -> Result<()> {
        if let Some((backend, package)) = package_str.split_once(':')
            && !backend.is_empty()
            && !package.is_empty()
        {
            config
                .packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .push(crate::config::kdl_modules::types::PackageEntry {
                    name: package.to_string(),
                });
        }
        Ok(())
    }
}

impl Default for BackendParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}
