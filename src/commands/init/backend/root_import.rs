use crate::error::{DeclarchError, Result};
use crate::project_identity;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Add backend import directly to declarch.kdl
///
/// Returns:
/// - Ok(true): Successfully added (or already present)
/// - Ok(false): backends {} block not found, manual import needed
/// - Err: Error during file operation
pub fn add_backend_to_declarch(declarch_kdl_path: &Path, backend_name: &str) -> Result<bool> {
    if !declarch_kdl_path.exists() {
        return Err(DeclarchError::Other(format!(
            "{} not found at {}",
            project_identity::CONFIG_FILE_BASENAME,
            declarch_kdl_path.display()
        )));
    }

    let content = fs::read_to_string(declarch_kdl_path)?;
    let import_path = format!("backends/{}.kdl", backend_name);

    let existing_pattern = format!(r#""{}""#, regex::escape(&import_path));
    if Regex::new(&existing_pattern)
        .map(|re| re.is_match(&content))
        .unwrap_or(false)
    {
        return Ok(true);
    }

    let backends_re = Regex::new(r#"(?m)^\s*backends\b"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex: {}", e)))?;

    if !backends_re.is_match(&content) {
        return Ok(false);
    }

    let backends_block_re = Regex::new(r#"(?m)^(\s*backends(?:\s+"[^"]*")?\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex: {}", e)))?;

    let import_line = format!(r#"    "{}""#, import_path);
    let new_content = backends_block_re
        .replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        })
        .to_string();

    fs::write(declarch_kdl_path, new_content)?;
    Ok(true)
}
