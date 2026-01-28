use crate::config::kdl_modules::types::PolicyConfig;
use crate::error::Result;
use kdl::KdlNode;

/// Parse policy block: policy { protected { linux systemd } orphans "keep" }
pub fn parse_policy(node: &KdlNode, policy: &mut PolicyConfig) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            match child_name {
                "protected" => {
                    // Protected packages: protected { linux systemd grub }
                    if let Some(grandchildren) = child.children() {
                        for gc in grandchildren.nodes() {
                            policy.protected.insert(gc.name().value().to_string());
                        }
                    }

                    // Also check for string arguments
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            policy.protected.insert(val.to_string());
                        }
                    }
                }
                "orphans" => {
                    // Orphans strategy: orphans "keep" | "remove" | "ask"
                    if let Some(val) = child.entries().first()
                        && let Some(strategy) = val.value().as_string()
                    {
                        policy.orphans = Some(strategy.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
