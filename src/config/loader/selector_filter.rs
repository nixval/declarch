use super::LoadSelectors;
use crate::error::{DeclarchError, Result};
use kdl::{KdlDocument, KdlNode};

pub(super) fn filter_content_by_selectors(
    content: &str,
    selectors: &LoadSelectors,
) -> Result<String> {
    if selectors.is_empty() {
        return Ok(content.to_string());
    }

    let doc: KdlDocument = content
        .parse()
        .map_err(|e: kdl::KdlError| DeclarchError::ConfigError(format!("Invalid KDL: {}", e)))?;

    let mut output = String::new();

    for node in doc.nodes() {
        match node.name().value() {
            "profile" => {
                if selectors
                    .profile
                    .as_ref()
                    .zip(selector_name(node))
                    .is_some_and(|(selected, current)| selected == &current)
                    && let Some(children) = node.children()
                {
                    for child in children.nodes() {
                        output.push_str(&child.to_string());
                        output.push('\n');
                    }
                }
            }
            "host" => {
                if selectors
                    .host
                    .as_ref()
                    .zip(selector_name(node))
                    .is_some_and(|(selected, current)| selected == &current)
                    && let Some(children) = node.children()
                {
                    for child in children.nodes() {
                        output.push_str(&child.to_string());
                        output.push('\n');
                    }
                }
            }
            _ => {
                output.push_str(&node.to_string());
                output.push('\n');
            }
        }
    }

    Ok(output)
}

fn selector_name(node: &KdlNode) -> Option<String> {
    node.entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .map(ToString::to_string)
}
