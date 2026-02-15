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
                        policy.orphans = Some(strategy.to_lowercase());
                    }
                }
                "require-backend" | "require_backend" => {
                    if let Some(value) = parse_first_bool(child) {
                        policy.require_backend = Some(value);
                    }
                }
                "forbid-hooks" | "forbid_hooks" => {
                    if let Some(value) = parse_first_bool(child) {
                        policy.forbid_hooks = Some(value);
                    }
                }
                "on-duplicate" | "on_duplicate" => {
                    if let Some(val) = child.entries().first()
                        && let Some(mode) = val.value().as_string()
                    {
                        let normalized = mode.to_lowercase();
                        if normalized == "warn" || normalized == "error" {
                            policy.on_duplicate = Some(normalized);
                        }
                    }
                }
                "on-conflict" | "on_conflict" => {
                    if let Some(val) = child.entries().first()
                        && let Some(mode) = val.value().as_string()
                    {
                        let normalized = mode.to_lowercase();
                        if normalized == "warn" || normalized == "error" {
                            policy.on_conflict = Some(normalized);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn parse_first_bool(node: &KdlNode) -> Option<bool> {
    let entry = node.entries().first()?;

    if let Some(v) = entry.value().as_bool() {
        return Some(v);
    }

    entry.value().as_string().and_then(|v| match v.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    })
}
