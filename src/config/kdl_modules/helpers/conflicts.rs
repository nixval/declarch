use crate::config::kdl_modules::types::ConflictEntry;
use crate::error::Result;
use kdl::KdlNode;

/// Parse conflicts block: conflicts { vim neovim }
pub fn parse_conflicts(node: &KdlNode, conflicts: &mut Vec<ConflictEntry>) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let mut packages = Vec::new();

            // Add the conflict block name
            packages.push(child.name().value().to_string());

            // Add all string arguments
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    packages.push(val.to_string());
                }
            }

            // Also add children names
            if let Some(grandchildren) = child.children() {
                for gc in grandchildren.nodes() {
                    packages.push(gc.name().value().to_string());
                }
            }

            conflicts.push(ConflictEntry {
                packages,
                condition: None, // Future: support conditional conflicts
            });
        }
    }

    Ok(())
}
