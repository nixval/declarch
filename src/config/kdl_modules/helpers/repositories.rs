use crate::error::Result;
use kdl::KdlNode;
use std::collections::HashMap;

/// Parse repositories: repos:aur { "https://..." }
pub fn parse_repositories(
    node: &KdlNode,
    repos: &mut HashMap<String, Vec<String>>,
) -> Result<()> {
    let mut repo_urls = Vec::new();

    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            repo_urls.push(val.to_string());
        }
    }

    // Extract from children node names
    if let Some(children) = node.children() {
        for child in children.nodes() {
            repo_urls.push(child.name().value().to_string());

            // Also check for string arguments in children
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    repo_urls.push(val.to_string());
                }
            }
        }
    }

    if !repo_urls.is_empty() {
        // The repository name is in the parent node, which is passed separately
        // For now, we'll use the first URL or a default name
        // This will be handled by the caller
        repos.insert("default".to_string(), repo_urls);
    }

    Ok(())
}
