//! User-defined backend parser
//!
//! Parses backend definitions from KDL configuration files,
//! allowing users to extend declarch with custom package managers.

use crate::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
use crate::error::{DeclarchError, Result};
use kdl::{KdlDocument, KdlNode};
use std::path::Path;

/// Load user-defined backends from backends.kdl (aggregator file)
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
        match node_name {
            "backend" => {
                let config = parse_backend_node(node)?;
                backends.push(config);
            }
            "import" => {
                // Handle top-level import statements: import "backends/name.kdl"
                if let Some(path_val) = node.entries().first()
                    .and_then(|e| e.value().as_string()) {
                    if let Ok(config_dir) = crate::utils::paths::config_dir() {
                        let import_path = config_dir.join(path_val);
                        match load_backend_file(&import_path) {
                            Ok(Some(config)) => {
                                backends.push(config);
                            }
                            Ok(None) => {
                                // File doesn't exist, skip
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load backend from '{}': {}", path_val, e);
                            }
                        }
                    }
                }
            }
            "imports" => {
                // Handle imports { ... } block
                // String entries like "backends/name.kdl" are entries (arguments) of the imports node
                // Check entries first
                for entry in node.entries() {
                    if let Some(path_val) = entry.value().as_string() {
                        if path_val.ends_with(".kdl") {
                            if let Ok(config_dir) = crate::utils::paths::config_dir() {
                                let import_path = config_dir.join(path_val);
                                match load_backend_file(&import_path) {
                                    Ok(Some(config)) => backends.push(config),
                                    Ok(None) => {}
                                    Err(e) => {
                                        eprintln!("Warning: Failed to load backend from '{}': {}", path_val, e);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Also check children (for import "path" style or other node types)
                if let Some(children) = node.children() {
                    for child in children.nodes() {
                        let child_name = child.name().value();
                        
                        // Handle import "path" nodes
                        if child_name == "import" {
                            if let Some(path_val) = child.entries().first()
                                .and_then(|e| e.value().as_string()) {
                                if let Ok(config_dir) = crate::utils::paths::config_dir() {
                                    let import_path = config_dir.join(path_val);
                                    match load_backend_file(&import_path) {
                                        Ok(Some(config)) => backends.push(config),
                                        Ok(None) => {}
                                        Err(e) => {
                                            eprintln!("Warning: Failed to load backend from '{}': {}", path_val, e);
                                        }
                                    }
                                }
                            }
                        }
                        // Handle bare string child nodes like "backends/name.kdl"
                        else if child_name.ends_with(".kdl") && child_name.contains('/') {
                            if let Ok(config_dir) = crate::utils::paths::config_dir() {
                                let import_path = config_dir.join(child_name);
                                match load_backend_file(&import_path) {
                                    Ok(Some(config)) => backends.push(config),
                                    Ok(None) => {}
                                    Err(e) => {
                                        eprintln!("Warning: Failed to load backend from '{}': {}", child_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(backends)
}

/// Parse a single backend from file content
/// 
/// Used for individual backend files in backends/ directory
pub fn parse_backend_file(content: &str) -> Result<Option<BackendConfig>> {
    let doc = KdlDocument::parse(content)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend file: {}", e)))?;

    for node in doc.nodes() {
        if node.name().value() == "backend" {
            return parse_backend_node(node).map(Some);
        }
    }

    Ok(None)
}

/// Load a single backend from file path
fn load_backend_file(path: &Path) -> Result<Option<BackendConfig>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read backend file: {}", e)))?;

    parse_backend_file(&content)
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
                "noconfirm" => parse_noconfirm(child, &mut config)?,
                "needs_sudo" | "sudo" => config.needs_sudo = parse_bool(child)?,
                "env" => parse_env(child, &mut config)?,
                "fallback" => parse_fallback(child, &mut config)?,
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
fn parse_list_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Extract command from argument
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "List command required. Usage: list \"command\" { ... }".to_string(),
            )
        })?;

    config.list_cmd = cmd.to_string();

    // Parse output format from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "format" => {
                    let format_str = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .ok_or_else(|| {
                            DeclarchError::Other(
                                "Format value required. Usage: format json|whitespace|tsv|regex"
                                    .to_string(),
                            )
                        })?;

                    config.list_format = match format_str {
                        "json" => OutputFormat::Json,
                        "whitespace" => OutputFormat::SplitWhitespace,
                        "tsv" => OutputFormat::TabSeparated,
                        "regex" => OutputFormat::Regex,
                        _ => {
                            return Err(DeclarchError::Other(format!(
                                "Unknown format '{}'. Valid: json, whitespace, tsv, regex",
                                format_str
                            )));
                        }
                    };
                }
                "json_path" => {
                    config.list_json_path = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_key" => {
                    config.list_name_key = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "version_key" => {
                    config.list_version_key = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
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
                                    config.list_regex_name_group = regex_child.entries().first().and_then(|entry| {
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
                                    config.list_regex_version_group = regex_child.entries().first().and_then(|entry| {
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
                    if config.list_regex.is_none() {
                        if let Some(pattern) = child
                            .entries()
                            .first()
                            .and_then(|entry| entry.value().as_string())
                        {
                            config.list_regex = Some(pattern.to_string());
                        }
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

/// Parse install command
fn parse_install_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.install_cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Install command required. Usage: install \"command\"".to_string())
        })?
        .to_string();

    Ok(())
}

/// Parse remove command
fn parse_remove_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.remove_cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Remove command required. Usage: remove \"command\"".to_string())
        })?
        .to_string();

    Ok(())
}

/// Parse noconfirm flag
fn parse_noconfirm(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.noconfirm_flag = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .map(|s| s.to_string());
    Ok(())
}

/// Parse search command with output format
fn parse_search_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Extract command from argument
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "Search command required. Usage: search \"command\" { ... }".to_string(),
            )
        })?;

    config.search_cmd = Some(cmd.to_string());

    // Parse output format from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "format" => {
                    let format_str = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .ok_or_else(|| {
                            DeclarchError::Other(
                                "Format value required. Usage: format json|whitespace|tsv|regex"
                                    .to_string(),
                            )
                        })?;

                    config.search_format = Some(match format_str {
                        "json" => OutputFormat::Json,
                        "whitespace" => OutputFormat::SplitWhitespace,
                        "tsv" => OutputFormat::TabSeparated,
                        "regex" => OutputFormat::Regex,
                        _ => {
                            return Err(DeclarchError::Other(format!(
                                "Unknown format '{}'. Valid: json, whitespace, tsv, regex",
                                format_str
                            )));
                        }
                    });
                }
                "json_path" => {
                    config.search_json_path = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_key" => {
                    config.search_name_key = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "version_key" => {
                    config.search_version_key = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "desc_key" => {
                    config.search_desc_key = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                // Nested json block for search: json { path "..." name_key "..." }
                "json" => {
                    if let Some(json_children) = child.children() {
                        for json_child in json_children.nodes() {
                            match json_child.name().value() {
                                "path" => {
                                    config.search_json_path = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_key" => {
                                    config.search_name_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "version_key" => {
                                    config.search_version_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "desc_key" => {
                                    config.search_desc_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "name_col" => {
                    config.search_name_col = child.entries().first().and_then(|entry| {
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
                "desc_col" => {
                    config.search_desc_col = child.entries().first().and_then(|entry| {
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
                // Regex pattern for search
                "regex" => {
                    config.search_regex = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.search_regex_name_group = child.entries().first().and_then(|entry| {
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
                "desc_group" => {
                    config.search_regex_desc_group = child.entries().first().and_then(|entry| {
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

    Ok(())
}

/// Parse fallback backend
fn parse_fallback(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.fallback = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .map(|s| s.to_string());
    Ok(())
}

/// Parse boolean value
/// Accepts both boolean literals (true/false) and strings ("true"/"false")
fn parse_bool(node: &KdlNode) -> Result<bool> {
    let entry = node.entries().first();
    
    // Try as boolean literal first
    if let Some(val) = entry.and_then(|e| e.value().as_bool()) {
        return Ok(val);
    }
    
    // Try as string "true" or "false"
    if let Some(s) = entry.and_then(|e| e.value().as_string()) {
        match s.to_lowercase().as_str() {
            "true" => return Ok(true),
            "false" => return Ok(false),
            _ => {}
        }
    }
    
    Err(DeclarchError::Other("Boolean value required. Usage: needs_sudo true or needs_sudo \"true\"".to_string()))
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
            "Backend 'list_cmd' cannot be empty".to_string(),
        ));
    }

    if config.install_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'install_cmd' cannot be empty".to_string(),
        ));
    }

    if config.remove_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'remove_cmd' cannot be empty".to_string(),
        ));
    }

    // Validate format-specific requirements
    match config.list_format {
        OutputFormat::Json => {
            if config.list_name_key.is_none() {
                return Err(DeclarchError::Other(
                    "JSON format requires 'name_key' to be specified in list block".to_string(),
                ));
            }
        }
        OutputFormat::Regex => {
            if config.list_regex.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'regex' to be specified in list block".to_string(),
                ));
            }
            if config.list_regex_name_group.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'name_group' to be specified".to_string(),
                ));
            }
        }
        OutputFormat::SplitWhitespace | OutputFormat::TabSeparated => {
            if config.list_name_col.is_none() {
                return Err(DeclarchError::Other(
                    "Whitespace/TSV format requires 'name_col' to be specified".to_string(),
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
        let config = BackendConfig {
            name: "test".to_string(),
            install_cmd: "test install".to_string(),
            remove_cmd: "test remove".to_string(),
            fallback: None,
            ..Default::default()
        };
        // list_cmd is empty

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_missing_name_key() {
        let config = BackendConfig {
            name: "test".to_string(),
            list_cmd: "test list".to_string(),
            list_format: OutputFormat::Json,
            install_cmd: "test install".to_string(),
            remove_cmd: "test remove".to_string(),
            fallback: None,
            ..Default::default()
        };
        // Missing name_key

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_regex_missing_regex() {
        let config = BackendConfig {
            name: "test".to_string(),
            list_cmd: "test list".to_string(),
            list_format: OutputFormat::Regex,
            install_cmd: "test install".to_string(),
            remove_cmd: "test remove".to_string(),
            fallback: None,
            ..Default::default()
        };
        // Missing regex

        let result = validate_backend_config(&config);
        assert!(result.is_err());
    }
}
