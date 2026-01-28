use crate::config::kdl_modules::types::ConflictEntry;
use crate::error::Result;
use kdl::KdlNode;

/// Parse conflicts block: conflicts { vim neovim } or conflicts vim neovim
pub fn parse_conflicts(node: &KdlNode, conflicts: &mut Vec<ConflictEntry>) -> Result<()> {
    let mut packages = Vec::new();

    // Extract from string arguments (inline format: conflicts vim neovim)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            packages.push(val.to_string());
        }
    }

    // Extract from children (block format: conflicts { vim neovim })
    if let Some(children) = node.children() {
        for child in children.nodes() {
            packages.push(child.name().value().to_string());
            // Also check for string arguments in child entries
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    packages.push(val.to_string());
                }
            }
        }
    }

    if !packages.is_empty() {
        conflicts.push(ConflictEntry {
            packages,
            condition: None, // Future: support conditional conflicts
        });
    }

    Ok(())
}
