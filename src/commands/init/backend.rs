//! Backend initialization logic
//!
//! Handles the `declarch init --backend <name>` command flow:
//! 1. Fetch backend definition from remote registry
//! 2. Validate and sanitize backend name
//! 3. Write backend file to `backends/<name>.kdl`
//! 4. Add import to `declarch.kdl` in `backends { ... }`

use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::{paths, remote};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Initialize a new backend configuration file
pub fn init_backend(backend_name: &str, force: bool) -> Result<()> {
    let root_dir = paths::config_dir()?;

    // Ensure declarch environment exists (auto-init if needed)
    super::root::ensure_environment()?;

    // Sanitize backend name
    let sanitized_name: String = backend_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if sanitized_name.is_empty() {
        return Err(DeclarchError::Other(
            "Invalid backend name. Use alphanumeric characters, hyphens, or underscores."
                .to_string(),
        ));
    }

    // Create backends directory
    let backends_dir = root_dir.join("backends");
    if !backends_dir.exists() {
        fs::create_dir_all(&backends_dir)?;
    }

    println!(
        "fetching '{}' from nixval/declarch-packages",
        sanitized_name
    );

    let backend_content = match remote::fetch_backend_content(&sanitized_name) {
        Ok(content) => content,
        Err(_) => {
            return Err(DeclarchError::Other(
                "not found, please check 'declarch init --list backends'".to_string(),
            ));
        }
    };

    // Validate KDL (warning only, can bypass with --force)
    if let Err(e) = super::validate_kdl(&backend_content, &format!("backend '{}'", sanitized_name))
        && !force
    {
        output::warning(&format!("{}", e));
        output::info("The backend may be malformed or incompatible with your declarch version.");
        output::info("You can still adopt it with --force, then edit the file manually.");

        if !output::prompt_yes_no("Continue with potentially invalid backend") {
            output::info("Cancelled. You can try a different backend or use --force to override.");
            return Ok(());
        }
    }

    // Parse and display meta information
    if let Ok(meta) = extract_backend_meta(&backend_content) {
        println!();
        if !meta.title.is_empty() && meta.title != "-" {
            println!("  Title:       {}", meta.title);
        }
        if !meta.description.is_empty() && meta.description != "-" {
            println!("  Description: {}", meta.description);
        }
        if !meta.maintainers.is_empty() {
            println!("  Maintainer:  {}", meta.maintainers.join(", "));
        } else if let Some(author) = &meta.author
            && author != "-"
        {
            println!("  Author:      {}", author);
        }
        if !meta.homepage.is_empty() && meta.homepage != "-" {
            println!("  Homepage:    {}", meta.homepage);
        }
        if let Some(guide) = &meta.installation_guide
            && guide != "-"
        {
            println!("  Install:     {}", guide);
        }
        if !meta.platforms.is_empty() {
            println!("  Platforms:   {}", meta.platforms.join(", "));
        }
        if !meta.requires.is_empty() {
            println!("  Requires:    {}", meta.requires.join(", "));
        }
        println!();
    }

    // Check if file already exists
    let backend_file = backends_dir.join(format!("{}.kdl", sanitized_name));
    if backend_file.exists() && !force {
        output::warning(&format!(
            "Backend file already exists: {}",
            backend_file.display()
        ));
        output::info("Use --force to overwrite.");
        return Ok(());
    }

    if !force
        && !output::prompt_yes_no(&format!(
            "Are you sure you want this '{}' being adopted",
            sanitized_name
        ))
    {
        output::info("Cancelled.");
        return Ok(());
    }

    fs::write(&backend_file, &backend_content)?;

    // Always import directly to declarch.kdl
    let declarch_kdl_path = root_dir.join("declarch.kdl");
    match add_backend_to_declarch(&declarch_kdl_path, &sanitized_name) {
        Ok(true) => {
            println!("Backend '{}' adopted.", sanitized_name);
        }
        Ok(false) => {
            println!(
                "Backend '{}' fetched. Add to declarch.kdl to use:",
                sanitized_name
            );
            output::info(&format!("backends {{\"backends/{}.kdl\"}}", sanitized_name));
        }
        Err(e) => {
            output::warning(&format!("Could not auto-import: {}", e));
            output::info(&format!(
                "Add manually to declarch.kdl: backends {{\"backends/{}.kdl\"}}",
                sanitized_name
            ));
        }
    }

    Ok(())
}

/// Add backend import directly to declarch.kdl
///
/// Returns:
/// - Ok(true): Successfully added (or already present)
/// - Ok(false): backends {} block not found, manual import needed
/// - Err: Error during file operation
pub fn add_backend_to_declarch(declarch_kdl_path: &Path, backend_name: &str) -> Result<bool> {
    if !declarch_kdl_path.exists() {
        return Err(DeclarchError::Other(format!(
            "declarch.kdl not found at {}",
            declarch_kdl_path.display()
        )));
    }

    let content = fs::read_to_string(declarch_kdl_path)?;
    let import_path = format!("backends/{}.kdl", backend_name);

    // Check if already imported
    let existing_pattern = format!(r#""{}""#, regex::escape(&import_path));
    if Regex::new(&existing_pattern)
        .map(|re| re.is_match(&content))
        .unwrap_or(false)
    {
        return Ok(true);
    }

    // Look for backends { ... } block
    let backends_re = Regex::new(r#"(?m)^\s*backends\b"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex: {}", e)))?;

    if !backends_re.is_match(&content) {
        return Ok(false);
    }

    // Add import in the backends block header line
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

/// Backend meta information extracted from KDL
#[derive(Debug, Default)]
pub struct BackendMeta {
    pub title: String,
    pub description: String,
    pub author: Option<String>,
    pub maintainers: Vec<String>,
    pub homepage: String,
    pub installation_guide: Option<String>,
    pub platforms: Vec<String>,
    pub requires: Vec<String>,
}

/// Extract meta information from backend KDL content
pub fn extract_backend_meta(content: &str) -> Result<BackendMeta> {
    let doc = kdl::KdlDocument::parse(content)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backend KDL: {}", e)))?;

    let mut meta = BackendMeta::default();

    for node in doc.nodes() {
        if node.name().value() != "backend" {
            continue;
        }

        if let Some(children) = node.children() {
            for child in children.nodes() {
                if child.name().value() != "meta" {
                    continue;
                }

                if let Some(meta_children) = child.children() {
                    for meta_node in meta_children.nodes() {
                        let name = meta_node.name().value();
                        match name {
                            "title" => {
                                meta.title = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "description" => {
                                meta.description = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "author" => {
                                meta.author = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .map(ToString::to_string);
                            }
                            // Support both old and new keys
                            "maintained" | "maintainer" => {
                                for entry in meta_node.entries() {
                                    if let Some(val) = entry.value().as_string() {
                                        meta.maintainers.push(val.to_string());
                                    }
                                }
                            }
                            "homepage" => {
                                meta.homepage = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("")
                                    .to_string();
                            }
                            "requires" => {
                                for entry in meta_node.entries() {
                                    if let Some(val) = entry.value().as_string() {
                                        meta.requires.push(val.to_string());
                                    }
                                }
                            }
                            "installation_guide" | "install-guide" => {
                                meta.installation_guide = meta_node
                                    .entries()
                                    .first()
                                    .and_then(|e| e.value().as_string())
                                    .map(ToString::to_string);
                            }
                            "platforms" => {
                                for entry in meta_node.entries() {
                                    if let Some(platform) = entry.value().as_string() {
                                        meta.platforms.push(platform.to_string());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        break;
    }

    meta.maintainers.sort();
    meta.maintainers.dedup();
    meta.platforms.sort();
    meta.platforms.dedup();
    meta.requires.sort();
    meta.requires.dedup();

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_backend_to_declarch_success() {
        let content = r#"meta {
    title "Host"
}

backends {
    "backends/aur.kdl"
}
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = add_backend_to_declarch(temp_file.path(), "flatpak").unwrap();
        assert!(result);

        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains(r#""backends/flatpak.kdl""#));
        assert!(!new_content.contains(r#"\"backends/flatpak.kdl\""#));
    }

    #[test]
    fn test_add_backend_to_declarch_no_block() {
        let content = r#"meta { title "Host" }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = add_backend_to_declarch(temp_file.path(), "flatpak").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_extract_backend_meta_maintainer_alias() {
        let content = r#"
backend "nala" {
    meta {
        title "Nala"
        maintained "declarch"
        maintainer "community-a" "community-b"
        homepage "https://example.com"
        install-guide "https://example.com/guide"
        requires "nala" "apt"
    }
    binary "nala"
    install "{binary} install {packages}"
}
"#;
        let meta = extract_backend_meta(content).unwrap();
        assert_eq!(meta.title, "Nala");
        assert_eq!(
            meta.maintainers,
            vec![
                "community-a".to_string(),
                "community-b".to_string(),
                "declarch".to_string(),
            ]
        );
        assert_eq!(
            meta.installation_guide,
            Some("https://example.com/guide".to_string())
        );
        assert_eq!(meta.requires, vec!["apt".to_string(), "nala".to_string()]);
    }
}
