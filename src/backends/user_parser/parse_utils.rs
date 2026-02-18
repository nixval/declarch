use crate::backends::config::BackendConfig;
use crate::error::{DeclarchError, Result};
use kdl::{KdlEntry, KdlNode};

pub(super) fn parse_supported_os(node: &KdlNode, config: &mut BackendConfig) {
    let mut values: Vec<String> = Vec::new();

    for entry in node.entries() {
        if entry.name().is_none()
            && let Some(val) = entry.value().as_string()
        {
            values.push(val.to_lowercase());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            if !child_name.is_empty() {
                values.push(child_name.to_lowercase());
            }
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    values.push(val.to_lowercase());
                }
            }
        }
    }

    values.sort();
    values.dedup();
    if !values.is_empty() {
        config.supported_os = Some(values);
    }
}

pub(super) fn get_entry_string(entry: &KdlEntry) -> Option<String> {
    if let Some(s) = entry.value().as_string() {
        return Some(s.to_string());
    }
    let val_str = entry.value().to_string();
    if !val_str.is_empty() {
        return Some(val_str);
    }
    None
}

pub(super) fn parse_bool(node: &KdlNode) -> Result<bool> {
    let entry = node.entries().first();

    if let Some(val) = entry.and_then(|e| e.value().as_bool()) {
        return Ok(val);
    }

    if let Some(s) = entry.and_then(|e| e.value().as_string()) {
        match s.to_lowercase().as_str() {
            "true" => return Ok(true),
            "false" => return Ok(false),
            _ => {}
        }
    }

    Err(DeclarchError::Other(
        "Boolean value required. Usage: needs_sudo true or needs_sudo \"true\"".to_string(),
    ))
}

pub(super) fn parse_env(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let mut env_map = std::collections::HashMap::new();

    for entry in node.entries() {
        if let Some(name) = entry.name() {
            let key = name.value();
            if let Some(value) = entry.value().as_string() {
                env_map.insert(key.to_string(), value.to_string());
            }
        } else if let Some(s) = entry.value().as_string() {
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
