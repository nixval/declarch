use crate::error::{DeclarchError, Result};

pub(super) fn normalize_kdl_output(mut content: String) -> Result<String> {
    content = content.replace("pkg{", "pkg {");

    let backend_block_re = regex::Regex::new(r"(?m)^([a-zA-Z][a-zA-Z0-9_-]*)\s*\r?\n\s*\{")
        .map_err(|e| DeclarchError::ConfigError(format!("Invalid regex: {}", e)))?;

    content = backend_block_re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{} {{", &caps[1])
        })
        .to_string();

    content = content
        .replace("}pkg", "}\npkg")
        .replace("}meta", "}\nmeta")
        .replace("}imports", "}\nimports")
        .replace("}hooks", "}\nhooks");

    let lines: Vec<&str> = content.lines().collect();
    let mut formatted_lines = Vec::new();
    let mut in_packages_block = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("pkg") && trimmed.contains('{') {
            in_packages_block = true;
            if line.contains("//") {
                formatted_lines.push(line.to_string());
            } else {
                formatted_lines.push(trimmed.to_string());
            }
            continue;
        }

        if trimmed == "}" {
            in_packages_block = false;
            formatted_lines.push(trimmed.to_string());
            continue;
        }

        if in_packages_block && !trimmed.is_empty() {
            formatted_lines.push(format!("  {}", trimmed));
            continue;
        }

        if !trimmed.is_empty() {
            formatted_lines.push(trimmed.to_string());
        } else {
            formatted_lines.push(String::new());
        }
    }

    Ok(formatted_lines.join("\n"))
}
