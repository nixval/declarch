use kdl::KdlNode;
use std::collections::HashMap;

/// Extract aliases: aliases { pipewire "pipewire-jack2" }
pub fn extract_aliases(node: &KdlNode, target: &mut HashMap<String, String>) {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let alias_name = child.name().value();

            // Get the actual package name (first string argument or child)
            if let Some(entry) = child.entries().first()
                && let Some(val) = entry.value().as_string()
            {
                target.insert(alias_name.to_string(), val.to_string());
            }
        }
    }
}

/// Parse alias entry and return (alias, actual) tuple
pub fn parse_alias_entry(node: &KdlNode) -> Option<(String, String)> {
    let alias_name = node.name().value();

    if let Some(entry) = node.entries().first()
        && let Some(val) = entry.value().as_string()
    {
        return Some((alias_name.to_string(), val.to_string()));
    }

    None
}
