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

    /// DEPRECATED: Parse packages node
    /// 
    /// This method is no longer used. Parsing is handled directly in parser.rs.
    pub fn parse_packages_node(&self, _node: &KdlNode, _config: &mut RawConfig) -> Result<()> {
        // No-op - parsing is handled by the generic parser
        Ok(())
    }

    /// DEPRECATED: Parse inline prefix
    pub fn parse_inline_prefix(&self, _package_str: &str, _config: &mut RawConfig) -> Result<()> {
        // No-op - parsing is handled by the generic parser
        Ok(())
    }
}

impl Default for BackendParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}
