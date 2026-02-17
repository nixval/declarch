use super::{BackendConfig, parse_backend_file};
use crate::error::{DeclarchError, Result};
use crate::ui;
use kdl::KdlNode;
use std::path::Path;

pub(super) fn collect_import_backends(node: &KdlNode) -> Result<Vec<BackendConfig>> {
    let mut backends = Vec::new();
    if let Some(path_val) = node.entries().first().and_then(|e| e.value().as_string())
        && let Ok(config_dir) = crate::utils::paths::config_dir()
    {
        let import_path = config_dir.join(path_val);
        if let Some(config) = load_backend_file(&import_path)? {
            backends.push(config);
        }
    }
    Ok(backends)
}

pub(super) fn collect_imports_block_backends(node: &KdlNode) -> Result<Vec<BackendConfig>> {
    let mut backends = Vec::new();

    for entry in node.entries() {
        if let Some(path_val) = entry.value().as_string()
            && path_val.ends_with(".kdl")
            && let Ok(config_dir) = crate::utils::paths::config_dir()
        {
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

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            if child_name == "import" {
                if let Some(path_val) = child.entries().first().and_then(|e| e.value().as_string())
                    && let Ok(config_dir) = crate::utils::paths::config_dir()
                {
                    let import_path = config_dir.join(path_val);
                    match load_backend_file(&import_path) {
                        Ok(Some(config)) => backends.push(config),
                        Ok(None) => {}
                        Err(e) => {
                            ui::warning(&format!(
                                "Failed to load backend from '{}': {}",
                                path_val, e
                            ));
                        }
                    }
                }
            } else if child_name.ends_with(".kdl")
                && child_name.contains('/')
                && let Ok(config_dir) = crate::utils::paths::config_dir()
            {
                let import_path = config_dir.join(child_name);
                match load_backend_file(&import_path) {
                    Ok(Some(config)) => backends.push(config),
                    Ok(None) => {}
                    Err(e) => {
                        ui::warning(&format!(
                            "Failed to load backend from '{}': {}",
                            child_name, e
                        ));
                    }
                }
            }
        }
    }

    Ok(backends)
}

fn load_backend_file(path: &Path) -> Result<Option<BackendConfig>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read backend file: {}", e)))?;

    parse_backend_file(&content)
}
