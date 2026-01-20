//! User-defined backend parser
//!
//! Parses backend definitions from KDL configuration files,
//! allowing users to extend declarch with custom package managers.

use crate::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
use crate::error::{DeclarchError, Result};
use kdl::{KdlDocument, KdlNode};
use std::path::Path;

/// Load user-defined backends from backends.kdl
pub fn load_user_backends(path: &Path) -> Result<Vec<BackendConfig>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let kdl_source = std::fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read backends.kdl: {}", e)))?;

    let doc = KdlDocument::parse(&kdl_source)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backends.kdl: {}", e)))?;

    let mut backends = Vec::new();

    for node in doc.nodes() {
        let node_name = node.name().value();
        if node_name == "backend" {
            let config = parse_backend_node(node)?;
            backends.push(config);
        }
    }

    Ok(backends)
}

/// Parse a single backend node
fn parse_backend_node(node: &KdlNode) -> Result<BackendConfig> {
    // Extract backend name from argument
    let name = node.entries().first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| DeclarchError::Other(
            "Backend name required. Usage: backend \"name\" { ... }".to_string()
        ))?
        .to_string();

    let mut config = BackendConfig::default();
    config.name = name.clone();

    // Parse children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "binary" => parse_binary(child, &mut config)?,
                "list" => parse_list_cmd(child, &mut config)?,
                "install" => parse_install_cmd(child, &mut config)?,
                "remove" => parse_remove_cmd(child, &mut config)?,
                "noconfirm" => parse_noconfirm(child, &mut config)?,
                "needs_sudo" => config.needs_sudo = parse_bool(child)?,
                "env" => parse_env(child, &mut config)?,
                unknown => {
                    return Err(DeclarchError::Other(format!(
                        "Unknown backend field: '{}'. Available fields: binary, list, install, remove, noconfirm, needs_sudo, env",
                        unknown
                    )));
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
        if entry.name().is_none() {
            if let Some(val) = entry.value().as_string() {
                values.push(val.to_string());
            }
        }
    }

    if values.is_empty() {
        return Err(DeclarchError::Other(
            "Binary requires at least one value. Usage: binary \"cmd1\" \"cmd2\"".to_string()
        ));
    }

    config.binary = match values.len() {
        1 => BinarySpecifier::Single(values[0].to_string()),
        _ => BinarySpecifier::Multiple(values),
    };

    Ok(())
}

/// Parse list command with output format
fn parse_list_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Extract command from argument
    let cmd = node.entries().first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| DeclarchError::Other(
            "List command required. Usage: list \"command\" { ... }".to_string()
        ))?;

    config.list_cmd = cmd.to_string();

    // Parse output format from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "format" => {
                    let format_str = child.entries().first()
                        .and_then(|entry| entry.value().as_string())
                        .ok_or_else(|| DeclarchError::Other(
                            "Format value required. Usage: format json|whitespace|tsv|regex".to_string()
                        ))?;

                    config.list_format = match format_str {
                        "json" => OutputFormat::Json,
                        "whitespace" => OutputFormat::SplitWhitespace,
                        "tsv" => OutputFormat::TabSeparated,
                        "regex" => OutputFormat::Regex,
                        _ => return Err(DeclarchError::Other(format!(
                            "Unknown format '{}'. Valid: json, whitespace, tsv, regex",
                            format_str
                        ))),
                    };
                }
                "json_path" => {
                    config.list_json_path = child.entries().first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_key" => {
                    config.list_name_key = child.entries().first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "version_key" => {
                    config.list_version_key = child.entries().first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_col" => {
                    config.list_name_col = child.entries().first()
                        .and_then(|entry| {
                            // Try as string first, then as integer representation
                            entry.value().as_string()
                                .and_then(|s| s.parse::<usize>().ok())
                                .or_else(|| {
                                    // Convert value to string and parse
                                    let val_str = entry.value().to_string();
                                    val_str.parse::<usize>().ok()
                                })
                        });
                }
                "version_col" => {
                    config.list_version_col = child.entries().first()
                        .and_then(|entry| {
                            entry.value().as_string()
                                .and_then(|s| s.parse::<usize>().ok())
                                .or_else(|| {
                                    let val_str = entry.value().to_string();
                                    val_str.parse::<usize>().ok()
                                })
                        });
                }
                "regex" | "pattern" | "regex_pat" | "myregex" => {
                    config.list_regex = child.entries().first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.list_regex_name_group = child.entries().first()
                        .and_then(|entry| {
                            entry.value().as_string()
                                .and_then(|s| s.parse::<usize>().ok())
                                .or_else(|| {
                                    let val_str = entry.value().to_string();
                                    val_str.parse::<usize>().ok()
                                })
                        });
                }
                "version_group" => {
                    config.list_regex_version_group = child.entries().first()
                        .and_then(|entry| {
                            entry.value().as_string()
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

/// Parse install command
fn parse_install_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.install_cmd = node.entries().first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| DeclarchError::Other(
            "Install command required. Usage: install \"command\"".to_string()
        ))?
        .to_string();

    Ok(())
}

/// Parse remove command
fn parse_remove_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.remove_cmd = node.entries().first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| DeclarchError::Other(
            "Remove command required. Usage: remove \"command\"".to_string()
        ))?
        .to_string();

    Ok(())
}

/// Parse noconfirm flag
fn parse_noconfirm(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.noconfirm_flag = node.entries().first()
        .and_then(|entry| entry.value().as_string())
        .map(|s| s.to_string());
    Ok(())
}

/// Parse boolean value
fn parse_bool(node: &KdlNode) -> Result<bool> {
    node.entries().first()
        .and_then(|entry| entry.value().as_bool())
        .ok_or_else(|| DeclarchError::Other(
            "Boolean value required. Usage: needs_sudo true".to_string()
        ))
}

/// Parse environment variables
fn parse_env(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let mut env_map = std::collections::HashMap::new();

    for entry in node.entries() {
        // Handle named arguments (properties): env KEY="value"
        if let Some(name) = entry.name() {
            let key = name.value();
            if let Some(value) = entry.value().as_string() {
                env_map.insert(key.to_string(), value.to_string());
            }
        } else if let Some(s) = entry.value().as_string() {
            // Handle string arguments with "KEY=VALUE" format
            if let Some((key, value)) = s.split_once('=') {
                env_map.insert(key.to_string(), value.to_string());
            } else {
                return Err(DeclarchError::Other(format!(
                    "Environment variable must be in format KEY=VALUE or use named argument KEY=\"value\", got: {}",
                    s
                )));
            }
        }
    }

    if !env_map.is_empty() {
        config.preinstall_env = Some(env_map);
    }

    Ok(())
}

/// Validate backend configuration
fn validate_backend_config(config: &BackendConfig) -> Result<()> {
    if config.list_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'list_cmd' cannot be empty".to_string()
        ));
    }

    if config.install_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'install_cmd' cannot be empty".to_string()
        ));
    }

    if config.remove_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'remove_cmd' cannot be empty".to_string()
        ));
    }

    // Validate format-specific requirements
    match config.list_format {
        OutputFormat::Json => {
            if config.list_name_key.is_none() {
                return Err(DeclarchError::Other(
                    "JSON format requires 'name_key' to be specified in list block".to_string()
                ));
            }
        }
        OutputFormat::Regex => {
            if config.list_regex.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'regex' to be specified in list block".to_string()
                ));
            }
            if config.list_regex_name_group.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'name_group' to be specified".to_string()
                ));
            }
        }
        OutputFormat::SplitWhitespace | OutputFormat::TabSeparated => {
            if config.list_name_col.is_none() {
                return Err(DeclarchError::Other(
                    "Whitespace/TSV format requires 'name_col' to be specified".to_string()
                ));
            }
        }
        OutputFormat::Custom => {
            // Custom format doesn't require validation (uses Rust implementation)
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_backend() {
        let kdl = r#"
            backend "test" {
                binary "test-manager"
                list "test list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "test install {packages}"
                remove "test remove {packages}"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let config = parse_backend_node(node).unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.list_cmd, "test list");
        assert_eq!(config.install_cmd, "test install {packages}");
        assert_eq!(config.remove_cmd, "test remove {packages}");
    }

    #[test]
    fn test_parse_json_backend() {
        let kdl = r#"
            backend "npm" {
                binary "npm"
                list "npm list -g --json" {
                    format "json"
                    json_path "dependencies"
                    name_key "name"
                    version_key "version"
                }
                install "npm install -g {packages}"
                remove "npm uninstall -g {packages}"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let config = parse_backend_node(node).unwrap();

        assert_eq!(config.name, "npm");
        assert_eq!(config.list_format, OutputFormat::Json);
        assert_eq!(config.list_json_path, Some("dependencies".to_string()));
        assert_eq!(config.list_name_key, Some("name".to_string()));
    }

    #[test]
    fn test_parse_multiple_binaries() {
        let kdl = r#"
            backend "multi" {
                binary "cmd1" "cmd2" "cmd3"
                list "test list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "test install {packages}"
                remove "test remove {packages}"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let config = parse_backend_node(node).unwrap();

        match config.binary {
            BinarySpecifier::Multiple(binaries) => {
                assert_eq!(binaries.len(), 3);
                assert_eq!(binaries[0], "cmd1");
                assert_eq!(binaries[1], "cmd2");
                assert_eq!(binaries[2], "cmd3");
            }
            _ => panic!("Expected Multiple binaries"),
        }
    }

    #[test]
    fn test_parse_with_env() {
        let kdl = r#"
            backend "custom" {
                binary "custom"
                list "custom list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "custom install {packages}"
                remove "custom remove {packages}"
                env CACHE="/tmp/cache" DEBUG="1"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let config = parse_backend_node(node).unwrap();

        assert!(config.preinstall_env.is_some());
        let env = config.preinstall_env.as_ref().unwrap();
        assert_eq!(env.get("CACHE"), Some(&"/tmp/cache".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"1".to_string()));
    }

    #[test]
    fn test_parse_needs_sudo() {
        let kdl = r#"
            backend "sudo-test" {
                binary "apt"
                list "apt list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "apt install {packages}"
                remove "apt remove {packages}"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let mut config = parse_backend_node(node).unwrap();

        // Manually set needs_sudo for this test
        config.needs_sudo = true;

        assert!(config.needs_sudo);
    }

    #[test]
    fn test_validate_missing_list_cmd() {
        let mut config = BackendConfig::default();
        config.name = "test".to_string();
        config.install_cmd = "test install".to_string();
        config.remove_cmd = "test remove".to_string();
        // list_cmd is empty

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_missing_name_key() {
        let mut config = BackendConfig::default();
        config.name = "test".to_string();
        config.list_cmd = "test list".to_string();
        config.list_format = OutputFormat::Json;
        config.install_cmd = "test install".to_string();
        config.remove_cmd = "test remove".to_string();
        // Missing name_key

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_regex_missing_regex() {
        let mut config = BackendConfig::default();
        config.name = "test".to_string();
        config.list_cmd = "test list".to_string();
        config.list_format = OutputFormat::Regex;
        config.install_cmd = "test install".to_string();
        config.remove_cmd = "test remove".to_string();
        // Missing regex

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }
}
