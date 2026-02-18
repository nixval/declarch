//! User-defined backend parser
//!
//! Parses backend definitions from KDL configuration files,
//! allowing users to extend declarch with custom package managers.

mod command_fields;
mod imports;
mod list_fields;
mod parse_utils;
mod search_fields;
mod validation;

#[cfg(test)]
use crate::backends::config::OutputFormat;
use crate::backends::config::{BackendConfig, BinarySpecifier};
use crate::error::{DeclarchError, Result};
use command_fields::{
    parse_cache_clean_cmd, parse_fallback, parse_install_cmd, parse_noconfirm, parse_remove_cmd,
    parse_update_cmd, parse_upgrade_cmd,
};
use imports::{collect_import_backends, collect_imports_block_backends};
use kdl::{KdlDocument, KdlNode};
use list_fields::parse_list_cmd;
use parse_utils::{parse_bool, parse_env, parse_supported_os};
use search_fields::{parse_search_cmd, parse_search_local_cmd};
use std::path::Path;
use validation::validate_backend_config;

/// Load user-defined backends from a KDL backend config file.
pub fn load_user_backends(path: &Path) -> Result<Vec<BackendConfig>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let kdl_source = std::fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read backend config: {}", e)))?;

    let doc = KdlDocument::parse(&kdl_source)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend config: {}", e)))?;

    let mut backends = Vec::new();

    for node in doc.nodes() {
        let node_name = node.name().value();
        match node_name {
            "backend" => {
                let config = parse_backend_node(node)?;
                backends.push(config);
            }
            "import" => {
                backends.extend(collect_import_backends(node)?);
            }
            "imports" => {
                backends.extend(collect_imports_block_backends(node)?);
            }
            _ => {}
        }
    }

    Ok(backends)
}

/// Parse a single backend from file content
///
/// Used for individual backend files in backends/ directory
pub(super) fn parse_backend_file(content: &str) -> Result<Option<BackendConfig>> {
    let doc = KdlDocument::parse(content)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend file: {}", e)))?;

    for node in doc.nodes() {
        if node.name().value() == "backend" {
            return parse_backend_node(node).map(Some);
        }
    }

    Ok(None)
}

/// Parse a single backend node
fn parse_backend_node(node: &KdlNode) -> Result<BackendConfig> {
    // Extract backend name from argument
    let name = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "Backend name required. Usage: backend \"name\" { ... }".to_string(),
            )
        })?
        .to_string();

    let mut config = BackendConfig {
        name: name.clone(),
        ..Default::default()
    };

    // Parse children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "binary" => parse_binary(child, &mut config)?,
                "list" => parse_list_cmd(child, &mut config)?,
                "install" => parse_install_cmd(child, &mut config)?,
                "remove" => parse_remove_cmd(child, &mut config)?,
                "search" => parse_search_cmd(child, &mut config)?,
                "search_local" => parse_search_local_cmd(child, &mut config)?,
                "update" => parse_update_cmd(child, &mut config)?,
                "cache_clean" => parse_cache_clean_cmd(child, &mut config)?,
                "upgrade" => parse_upgrade_cmd(child, &mut config)?,
                "noconfirm" => parse_noconfirm(child, &mut config)?,
                "needs_sudo" | "sudo" => config.needs_sudo = parse_bool(child)?,
                "prefer_list_for_local_search" => {
                    config.prefer_list_for_local_search = parse_bool(child)?
                }
                "env" => parse_env(child, &mut config)?,
                "fallback" => parse_fallback(child, &mut config)?,
                "platforms" | "supported_os" | "os" => parse_supported_os(child, &mut config),
                _ => {
                    // Ignore unknown fields for forward compatibility
                }
            }
        }
    }

    // Validate required fields
    validate_backend_config(&config)?;

    Ok(config)
}

/// Parse binary specification
fn parse_binary(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let mut values = Vec::new();
    for entry in node.entries() {
        // Only use positional arguments (entries without a name)
        if entry.name().is_none()
            && let Some(val) = entry.value().as_string()
        {
            values.push(val.to_string());
        }
    }

    if values.is_empty() {
        return Err(DeclarchError::Other(
            "Binary requires at least one value. Usage: binary \"cmd1\" \"cmd2\"".to_string(),
        ));
    }

    config.binary = match values.len() {
        1 => BinarySpecifier::Single(values[0].to_string()),
        _ => BinarySpecifier::Multiple(values),
    };

    Ok(())
}

#[cfg(test)]
mod tests;
