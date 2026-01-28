use crate::config::kdl_modules::types::ConfigMeta;
use crate::error::Result;
use kdl::KdlNode;

/// Parse meta block: meta { description "..." author "..." version "..." }
pub fn parse_meta_block(node: &KdlNode, meta: &mut ConfigMeta) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            match child_name {
                "title" => {
                    if let Some(val) = super::get_first_string(child) {
                        meta.title = Some(val);
                    }
                }
                "description" => {
                    if let Some(val) = super::get_first_string(child) {
                        meta.description = Some(val);
                    }
                }
                "author" => {
                    if let Some(val) = super::get_first_string(child) {
                        meta.author = Some(val);
                    }
                }
                "version" => {
                    if let Some(val) = super::get_first_string(child) {
                        meta.version = Some(val);
                    }
                }
                "tags" => {
                    // tags can be multiple: tags ["workstation" "gaming"]
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            meta.tags.push(val.to_string());
                        }
                    }
                }
                "url" => {
                    if let Some(val) = super::get_first_string(child) {
                        meta.url = Some(val);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Get first string value from a KDL node
pub fn get_first_string(node: &KdlNode) -> Option<String> {
    if let Some(entry) = node.entries().first()
        && let Some(val) = entry.value().as_string()
    {
        return Some(val.to_string());
    }
    None
}
