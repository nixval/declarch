use crate::config::kdl_modules::types::PackageEntry;
use kdl::KdlNode;

/// Extract packages from a node and add them to a target vector
///
/// Handles:
/// - String arguments: `pkg "bat" "exa"`
/// - Children node names: `pkg { bat exa }`
/// - Mixed: `pkg "bat" { exa }`
pub fn extract_packages_to(node: &KdlNode, target: &mut Vec<PackageEntry>) {
    // Extract from string arguments of this node
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(PackageEntry {
                name: val.to_string(),
            });
        }
    }

    // Extract from children node names
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            let child_entries: Vec<_> = child
                .entries()
                .iter()
                .filter_map(|e| e.value().as_string())
                .collect();

            if child_entries.is_empty() {
                // No string arguments, just the node name
                target.push(PackageEntry {
                    name: child_name.to_string(),
                });
            } else {
                // Has string arguments - push node name AND all arguments
                // First, push the node name
                target.push(PackageEntry {
                    name: child_name.to_string(),
                });
                // Then push all string arguments
                for entry in &child_entries {
                    target.push(PackageEntry {
                        name: entry.to_string(),
                    });
                }
            }
        }
    }
}

/// Extract mixed string values from a node (both entries and children names)
pub fn extract_mixed_values(node: &KdlNode, target: &mut Vec<String>) {
    // Extract from children node names
    if let Some(children) = node.children() {
        for child in children.nodes() {
            target.push(child.name().value().to_string());
        }
    }

    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }
}

/// Extract mixed string values and return as Vec
pub fn extract_mixed_values_return(node: &KdlNode) -> Vec<String> {
    let mut values = Vec::new();
    extract_mixed_values(node, &mut values);
    values
}

/// Extract only string values from node entries (not children)
pub fn extract_strings(node: &KdlNode, target: &mut Vec<String>) {
    // First, extract from entries (key-value pairs on same line)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    // Also extract from children nodes (nodes inside braces)
    // This handles cases like: imports { "file1.kdl" "file2.kdl" }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            // Get the string value from child node name
            let child_name = child.name().value();
            // Skip comment nodes (they start with //)
            if !child_name.starts_with("//") {
                target.push(child_name.to_string());
            }
        }
    }
}
