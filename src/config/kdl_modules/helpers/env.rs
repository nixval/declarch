use crate::error::Result;
use kdl::KdlNode;
use std::collections::HashMap;

/// Parse environment variables: env { "EDITOR=nvim" } or env:paru { "MAKEFLAGS=-j4" }
pub fn parse_env_vars(
    node: &KdlNode,
    env: &mut HashMap<String, Vec<String>>,
    backend: Option<&str>,
) -> Result<()> {
    // Check for colon syntax: env:paru
    let backend_name = if let Some((_, b)) = node.name().value().split_once(':') {
        b.to_string()
    } else {
        backend.unwrap_or("global").to_string()
    };

    let mut vars = Vec::new();

    // Extract from string arguments (format: "EDITOR=nvim")
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            vars.push(val.to_string());
        }
    }

    // Extract from named arguments (format: EDITOR="nvim")
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            let key = name.value();
            if let Some(val) = entry.value().as_string() {
                // Only format as key=value if not already in key=value format
                if !key.is_empty()
                    && !key.contains('(')
                    && !vars.contains(&format!("{}={}", key, val))
                {
                    vars.push(format!("{}={}", key, val));
                }
            }
        }
    }

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            // Child name could be the key, value in arguments
            let key = child.name().value();
            if let Some(val) = super::meta::get_first_string(child) {
                vars.push(format!("{}={}", key, val));
            } else if let Some(val) = child.entries().first()
                && let Some(val) = val.value().as_string()
            {
                vars.push(format!("{}={}", key, val));
            }
        }
    }

    if !vars.is_empty() {
        env.insert(backend_name, vars);
    }

    Ok(())
}
