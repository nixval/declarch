//! KDL content parser
//!
//! Main parsing logic for KDL configuration files.
//!
//! In v0.6+, this uses a fully generic approach where all packages are
//! stored in unified storage (packages_by_backend).

mod ast_scan;
mod semantic_mapping;

use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

pub fn parse_backend_options(
    node: &KdlNode,
    options: &mut std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) -> Result<()> {
    semantic_mapping::parse_backend_options(node, options)
}

/// Parse KDL content into RawConfig
///
/// This is the main entry point for parsing KDL configuration files.
/// Uses fully unified package storage - no backend-specific fields.
pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    parse_kdl_content_with_path(content, None)
}

/// Parse KDL content with file path for better error reporting
///
/// Shows detailed error messages with line numbers, visual indicators,
/// and helpful hints similar to Rust compiler errors.
pub fn parse_kdl_content_with_path(content: &str, file_path: Option<&str>) -> Result<RawConfig> {
    let doc = ast_scan::parse_document_with_error_reporting(content, file_path)?;
    let mut config = RawConfig::default();

    for node in doc.nodes() {
        apply_node(node, &mut config)?;
    }

    Ok(config)
}

fn apply_node(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    semantic_mapping::map_node_into_config(node, config)
}
