use crate::error::{DeclarchError, Result};

use super::BackendMeta;

pub(super) fn extract_backend_meta_impl(content: &str) -> Result<BackendMeta> {
    let doc = kdl::KdlDocument::parse(content)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend KDL: {}", e)))?;

    let mut meta = BackendMeta::default();

    for node in doc.nodes() {
        if node.name().value() != "backend" {
            continue;
        }

        if let Some(children) = node.children() {
            for child in children.nodes() {
                if child.name().value() != "meta" {
                    continue;
                }

                if let Some(meta_children) = child.children() {
                    for meta_node in meta_children.nodes() {
                        let name = meta_node.name().value();
                        match name {
                            "title" => {
                                meta.title = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "description" => {
                                meta.description = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "author" => {
                                meta.author = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .map(ToString::to_string);
                            }
                            "kdl-maintainer" | "maintained" | "maintainer" => {
                                for entry in meta_node.entries() {
                                    if let Some(val) = entry.value().as_string() {
                                        meta.maintainers.push(val.to_string());
                                    }
                                }
                            }
                            "homepage" => {
                                meta.homepage = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "requires" => {
                                for entry in meta_node.entries() {
                                    if let Some(val) = entry.value().as_string() {
                                        meta.requires.push(val.to_string());
                                    }
                                }
                            }
                            "install-guide" | "installation_guide" => {
                                meta.installation_guide = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .map(ToString::to_string);
                            }
                            "platforms" => {
                                for entry in meta_node.entries() {
                                    if let Some(platform) = entry.value().as_string() {
                                        meta.platforms.push(platform.to_string());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        break;
    }

    meta.maintainers.sort();
    meta.maintainers.dedup();
    meta.platforms.sort();
    meta.platforms.dedup();
    meta.requires.sort();
    meta.requires.dedup();

    Ok(meta)
}
