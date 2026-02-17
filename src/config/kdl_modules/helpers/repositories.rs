use crate::error::Result;
use kdl::KdlNode;
use std::collections::HashMap;

/// Parse package sources: repos:paru { "https://..." }
pub fn parse_repositories(node: &KdlNode, repos: &mut HashMap<String, Vec<String>>) -> Result<()> {
    // Check for colon syntax: repos:paru
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified - skip
        return Ok(());
    };

    let mut repo_urls = Vec::new();

    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            repo_urls.push(val.to_string());
        }
    }

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            repo_urls.push(child.name().value().to_string());
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    repo_urls.push(val.to_string());
                }
            }
        }
    }

    if !repo_urls.is_empty() {
        let entry = repos.entry(backend_name).or_default();
        for url in repo_urls {
            let trimmed = url.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !entry.iter().any(|existing| existing == trimmed) {
                entry.push(trimmed.to_string());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
