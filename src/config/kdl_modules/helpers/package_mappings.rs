use kdl::KdlNode;
use std::collections::HashMap;

/// Extract package mappings: aliases-pkg config_name actual_name
pub fn extract_aliases(node: &KdlNode, target: &mut HashMap<String, String>) {
    // Case 1: Inline format: aliases-pkg config_name actual_name
    let entries: Vec<_> = node
        .entries()
        .iter()
        .filter_map(|e| e.value().as_string())
        .collect();

    if entries.len() == 2 {
        // Extract: aliases-pkg config_name actual_name
        target.insert(entries[0].to_string(), entries[1].to_string());
    }

    // Case 2: Children format
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_entries: Vec<_> = child
                .entries()
                .iter()
                .filter_map(|e| e.value().as_string())
                .collect();

            if child_entries.len() == 2 {
                target.insert(child_entries[0].to_string(), child_entries[1].to_string());
            } else if child_entries.len() == 1 {
                // Support: node-name target-name
                let config_name = child.name().value();
                target.insert(config_name.to_string(), child_entries[0].to_string());
            }
        }
    }
}

/// Parse package mapping entry and return (config_name, actual_name) tuple
pub fn parse_alias_entry(node: &KdlNode) -> Option<(String, String)> {
    let config_name = node.name().value();

    if let Some(entry) = node.entries().first()
        && let Some(val) = entry.value().as_string()
    {
        return Some((config_name.to_string(), val.to_string()));
    }

    None
}
