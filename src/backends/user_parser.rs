//! User-defined backend parser
//!
//! Parses backend definitions from KDL configuration files,
//! allowing users to extend declarch with custom package managers.

mod command_fields;
mod imports;
mod parse_utils;
mod search_fields;
mod validation;

use crate::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
use crate::error::{DeclarchError, Result};
use command_fields::{
    parse_cache_clean_cmd, parse_fallback, parse_install_cmd, parse_noconfirm, parse_remove_cmd,
    parse_update_cmd, parse_upgrade_cmd,
};
use imports::{collect_import_backends, collect_imports_block_backends};
use kdl::{KdlDocument, KdlNode};
use parse_utils::{get_entry_string, parse_bool, parse_env, parse_supported_os};
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

/// Parse list command with output format
/// Accepts "-" as sentinel value to explicitly disable (no warning)
fn parse_list_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Extract command from argument
    let cmd = node
        .entries()
        .first()
        .and_then(get_entry_string)
        .ok_or_else(|| {
            DeclarchError::Other(
                "List command required. Usage: list \"command\" { ... }".to_string(),
            )
        })?;

    // "-" means explicitly disabled (no warning)
    if cmd != "-" {
        config.list_cmd = Some(cmd.to_string());
    }

    // Parse output format from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "format" => {
                    let format_str = child
                        .entries()
                        .first()
                        .and_then(get_entry_string)
                        .ok_or_else(|| {
                            DeclarchError::Other(
                                "Format value required. Usage: format json|whitespace|tsv|regex"
                                    .to_string(),
                            )
                        })?;

                    config.list_format = match format_str.as_str() {
                        "json" => OutputFormat::Json,
                        "json_lines" | "jsonl" | "ndjson" => OutputFormat::JsonLines,
                        "npm_json" => OutputFormat::NpmJson,
                        "json_object_keys" => OutputFormat::JsonObjectKeys,
                        "whitespace" => OutputFormat::SplitWhitespace,
                        "tsv" => OutputFormat::TabSeparated,
                        "regex" => OutputFormat::Regex,
                        _ => {
                            return Err(DeclarchError::Other(format!(
                                "Unknown format '{}'. Valid: json, json_lines, npm_json, json_object_keys, whitespace, tsv, regex",
                                format_str
                            )));
                        }
                    };
                }
                "json_path" => {
                    config.list_json_path = child.entries().first().and_then(get_entry_string);
                }
                "name_key" => {
                    config.list_name_key = child.entries().first().and_then(get_entry_string);
                }
                "version_key" => {
                    config.list_version_key = child.entries().first().and_then(get_entry_string);
                }
                // Nested json block: json { path "..." name_key "..." version_key "..." }
                "json" => {
                    if let Some(json_children) = child.children() {
                        for json_child in json_children.nodes() {
                            match json_child.name().value() {
                                "path" => {
                                    config.list_json_path = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_key" => {
                                    config.list_name_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "version_key" => {
                                    config.list_version_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "desc_key" => {
                                    // Store somewhere if needed for search
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "name_col" => {
                    config.list_name_col = child.entries().first().and_then(|entry| {
                        // Try as string first, then as integer representation
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                // Convert value to string and parse
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                "version_col" => {
                    config.list_version_col = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                // Handle nested regex block: regex { pattern "..." name_group 1 }
                "regex" => {
                    // Check if regex has children (nested format)
                    if let Some(regex_children) = child.children() {
                        for regex_child in regex_children.nodes() {
                            match regex_child.name().value() {
                                "pattern" => {
                                    config.list_regex = regex_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_group" => {
                                    config.list_regex_name_group =
                                        regex_child.entries().first().and_then(|entry| {
                                            entry
                                                .value()
                                                .as_string()
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .or_else(|| {
                                                    let val_str = entry.value().to_string();
                                                    val_str.parse::<usize>().ok()
                                                })
                                        });
                                }
                                "version_group" => {
                                    config.list_regex_version_group =
                                        regex_child.entries().first().and_then(|entry| {
                                            entry
                                                .value()
                                                .as_string()
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .or_else(|| {
                                                    let val_str = entry.value().to_string();
                                                    val_str.parse::<usize>().ok()
                                                })
                                        });
                                }
                                _ => {}
                            }
                        }
                    }
                    // Also support flat format: regex "pattern"
                    if config.list_regex.is_none()
                        && let Some(pattern) = child
                            .entries()
                            .first()
                            .and_then(|entry| entry.value().as_string())
                    {
                        config.list_regex = Some(pattern.to_string());
                    }
                }
                // Flat format (legacy support)
                "pattern" | "regex_pat" | "myregex" => {
                    config.list_regex = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.list_regex_name_group = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                "version_group" => {
                    config.list_regex_version_group = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                _ => {
                    // Ignore unknown fields in list block
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
