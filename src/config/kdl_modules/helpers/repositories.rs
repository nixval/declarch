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
mod tests {
    use super::*;
    use kdl::KdlDocument;

    fn first_node(input: &str) -> KdlNode {
        let doc: KdlDocument = input.parse().expect("valid kdl");
        doc.nodes().first().expect("node exists").clone()
    }

    #[test]
    fn parse_repositories_merges_without_duplicates() {
        let mut repos = HashMap::new();
        let node1 = first_node(r#"repos:paru "core" "extra""#);
        let node2 = first_node(r#"repos:paru "extra" "multilib""#);

        parse_repositories(&node1, &mut repos).expect("parse node1");
        parse_repositories(&node2, &mut repos).expect("parse node2");

        let paru = repos.get("paru").expect("paru repos present");
        assert_eq!(paru, &vec!["core".to_string(), "extra".to_string(), "multilib".to_string()]);
    }

    #[test]
    fn parse_repositories_ignores_non_colon_syntax() {
        let mut repos = HashMap::new();
        let node = first_node(r#"repos "core""#);

        parse_repositories(&node, &mut repos).expect("parse node");

        assert!(repos.is_empty());
    }
}
