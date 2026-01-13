use kdl::{KdlDocument, KdlNode};
use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RawConfig {
    pub imports: Vec<String>,
    pub packages: Vec<String>,
    pub excludes: Vec<String>,
    /// Package aliases: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub aliases: HashMap<String, String>,
}

pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse()?;

    let mut config = RawConfig {
        imports: vec![],
        packages: vec![],
        excludes: vec![],
        aliases: HashMap::new(),
    };

    for node in doc.nodes() {
        match node.name().value() {
            "import" | "imports" => {
                extract_strings(node, &mut config.imports);
            },
            "packages" | "package" => {
                extract_mixed_values(node, &mut config.packages);
            },
            "exclude" | "excludes" => {
                extract_mixed_values(node, &mut config.excludes);
            },
            "aliases-pkg" | "alias-pkg" => {
                extract_aliases(node, &mut config.aliases);
            },
            _ => {}
        }
    }

    Ok(config)
}

fn extract_mixed_values(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                 if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

fn extract_strings(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

/// Extract package aliases from KDL node
///
/// Supported syntax:
/// ```kdl
/// aliases-pkg pipewire pipewire-jack2
/// aliases-pkg {
///     python-poetry python-poetry-core
///     firefox firefox-beta
/// }
/// ```
fn extract_aliases(node: &KdlNode, target: &mut HashMap<String, String>) {
    // Case 1: Inline format: aliases-pkg name1 name2
    let entries: Vec<_> = node.entries().iter()
        .filter_map(|e| e.value().as_string())
        .collect();

    if entries.len() == 2 {
        // aliases-pkg config_name actual_name
        target.insert(entries[0].to_string(), entries[1].to_string());
    }

    // Case 2: Children format
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_entries: Vec<_> = child.entries().iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aliases_inline() {
        let kdl = r#"
            aliases-pkg pipewire pipewire-jack2
            aliases-pkg python-poetry python-poetry-core
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
        assert_eq!(config.aliases.get("python-poetry"), Some(&"python-poetry-core".to_string()));
    }

    #[test]
    fn test_parse_aliases_block() {
        let kdl = r#"
            aliases-pkg {
                pipewire pipewire-jack2
                python-poetry python-poetry-core
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
        assert_eq!(config.aliases.get("python-poetry"), Some(&"python-poetry-core".to_string()));
    }

    #[test]
    fn test_parse_mixed_config() {
        let kdl = r#"
            packages {
                neovim
                hyprland
            }

            aliases-pkg pipewire pipewire-jack2

            excludes bad-package
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.aliases.len(), 1);
        assert_eq!(config.excludes.len(), 1);
        assert_eq!(config.aliases.get("pipewire"), Some(&"pipewire-jack2".to_string()));
    }

    #[test]
    fn test_parse_empty_aliases() {
        let kdl = r#"
            packages neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.aliases.is_empty());
    }
}
