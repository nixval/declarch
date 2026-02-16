use crate::config::kdl_modules::types::ProjectMetadata;
use crate::error::Result;
use kdl::KdlNode;

/// Parse meta block: meta { description "..." author "..." version "..." }
pub fn parse_meta_block(node: &KdlNode, project_metadata: &mut ProjectMetadata) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            match child_name {
                "title" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.title = Some(val);
                    }
                }
                "description" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.description = Some(val);
                    }
                }
                "author" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.author = Some(val);
                    }
                }
                "maintainer" | "maintained" => {
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            project_metadata.maintainers.push(val.to_string());
                        }
                    }
                }
                "version" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.version = Some(val);
                    }
                }
                "tags" => {
                    // tags can be multiple: tags ["workstation" "gaming"]
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            project_metadata.tags.push(val.to_string());
                        }
                    }
                }
                "url" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.url = Some(val);
                    }
                }
                "homepage" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.homepage = Some(val);
                    }
                }
                "license" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.license = Some(val);
                    }
                }
                "platforms" => {
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            project_metadata.platforms.push(val.to_string());
                        }
                    }
                }
                "requires" => {
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            project_metadata.requires.push(val.to_string());
                        }
                    }
                }
                "installation_guide" | "install-guide" => {
                    if let Some(val) = super::get_first_string(child) {
                        project_metadata.installation_guide = Some(val);
                    }
                }
                _ => {}
            }
        }
    }
    project_metadata.maintainers.sort();
    project_metadata.maintainers.dedup();
    project_metadata.platforms.sort();
    project_metadata.platforms.dedup();
    project_metadata.requires.sort();
    project_metadata.requires.dedup();
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
