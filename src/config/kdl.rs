use kdl::{KdlDocument, KdlNode};
use crate::utils::errors::{DeclarchError, Result};

#[derive(Debug, Clone)]
pub struct RawConfig {
    pub imports: Vec<String>,
    pub packages: Vec<String>,
    pub excludes: Vec<String>,
}

pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse()
        .map_err(|e: kdl::KdlError| DeclarchError::KdlParseError { reason: e.to_string() })?;

    let mut config = RawConfig {
        imports: vec![],
        packages: vec![],
        excludes: vec![],
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
