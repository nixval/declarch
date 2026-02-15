//! Config Editor Module
//!
//! This module provides functionality to programmatically edit KDL configuration files.
//! It's used by the `install` command to add packages to config files.

use crate::constants::CONFIG_EXTENSION;
use crate::error::{DeclarchError, Result};
use crate::utils::paths;
use kdl::{KdlDocument, KdlNode};
use std::fs;
use std::path::{Path, PathBuf};

/// Detect the default backend based on the Linux distribution
fn detect_default_backend() -> &'static str {
    // Check /etc/os-release for distro identification
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let id = content
            .lines()
            .find(|line| line.starts_with("ID="))
            .and_then(|line| line.strip_prefix("ID="))
            .map(|s| s.trim_matches('"'));

        match id {
            Some("debian") | Some("ubuntu") | Some("linuxmint") | Some("pop") => "apt",
            Some("fedora") | Some("rhel") | Some("centos") | Some("rocky") | Some("almalinux") => {
                "dnf"
            }
            Some("opensuse") | Some("opensuse-tumbleweed") | Some("suse") => "zypper",
            Some("arch") | Some("manjaro") | Some("endeavouros") | Some("cachyos") => "aur",
            _ => "aur", // Default to aur for unknown distros (most likely Arch-based)
        }
    } else {
        // Fallback: check if pacman exists
        if Path::new("/usr/bin/pacman").exists() {
            "aur"
        } else if Path::new("/usr/bin/apt").exists() {
            "apt"
        } else if Path::new("/usr/bin/dnf").exists() {
            "dnf"
        } else {
            "aur" // Ultimate fallback
        }
    }
}

/// Result of editing a config file
#[derive(Debug)]
pub struct ModuleEdit {
    pub file_path: PathBuf,
    pub packages_added: Vec<String>,
    pub created_new_file: bool,
    pub backup_path: Option<PathBuf>,
}

/// Config editor for programmatically editing KDL files
pub struct ConfigEditor;

impl Default for ConfigEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigEditor {
    /// Create a new config editor
    pub fn new() -> Self {
        Self
    }

    /// Add a package to the appropriate config file
    ///
    /// # Arguments
    /// * `package` - Package name (without backend prefix)
    /// * `backend` - Optional backend name (e.g., "soar", "npm")
    /// * `module` - Optional module name (e.g., "base", "linux/notes")
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use declarch::config::editor::ConfigEditor;
    /// # use declarch::error::Result;
    /// # fn main() -> Result<()> {
    /// let editor = ConfigEditor::new();
    ///
    /// // Add to default module (others.kdl)
    /// editor.add_package("hyprland", None, None)?;
    ///
    /// // Add to backend-specific block
    /// editor.add_package("bat", Some("soar"), None)?;
    ///
    /// // Add to specific module
    /// editor.add_package("nano", None, Some("base"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_package(
        &self,
        package: &str,
        backend: Option<&str>,
        module: Option<&str>,
    ) -> Result<ModuleEdit> {
        // Determine target file
        let target_file = self.resolve_module_path(module)?;

        // Create backup if file exists
        let backup_path = if target_file.exists() {
            Some(backup_kdl_file(&target_file)?)
        } else {
            None
        };

        // Load or create file
        let created_new_file = if !target_file.exists() {
            self.create_default_module(&target_file)?;
            true
        } else {
            false
        };

        // Parse existing content
        let content = fs::read_to_string(&target_file)?;

        // Add package to content
        let (updated_content, packages_added) =
            self.add_package_to_content(&content, package, backend)?;

        // Save updated content
        fs::write(&target_file, updated_content)?;

        Ok(ModuleEdit {
            file_path: target_file,
            packages_added,
            created_new_file,
            backup_path,
        })
    }

    /// Resolve module path from module name
    ///
    /// Examples:
    /// - None → modules/others.kdl
    /// - "base" → modules/base.kdl
    /// - "linux/notes" → modules/linux/notes.kdl
    fn resolve_module_path(&self, module: Option<&str>) -> Result<PathBuf> {
        let modules_dir = paths::modules_dir()?;

        let target_file = match module {
            Some(mod_name) => {
                // Split on "/" to handle nested paths
                let parts: Vec<&str> = mod_name.split('/').collect();

                let file_name = format!(
                    "{}.{}",
                    parts
                        .last()
                        .ok_or_else(|| DeclarchError::Other("Invalid module path".to_string()))?,
                    CONFIG_EXTENSION
                );
                let dir_path = parts
                    .iter()
                    .take(parts.len() - 1)
                    .fold(modules_dir, |acc, part| acc.join(part));

                // Ensure parent directory exists
                if !dir_path.exists() {
                    fs::create_dir_all(&dir_path).map_err(|e| {
                        DeclarchError::Other(format!("Failed to create directory: {}", e))
                    })?;
                }

                dir_path.join(file_name)
            }
            None => modules_dir.join(format!("others.{}", CONFIG_EXTENSION)),
        };

        Ok(target_file)
    }

    /// Create a default module file with proper structure
    fn create_default_module(&self, path: &PathBuf) -> Result<()> {
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DeclarchError::Other("Invalid module name".to_string()))?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DeclarchError::Other(format!("Failed to create module directory: {}", e))
            })?;
        }

        let default_content = format!(
            "// Declarch module: {}\n\
            // Generated by declarch install command\n\
            \n\
            pkg {{\n\
              // Add packages here\n\
            }}\n",
            module_name
        );

        fs::write(path, default_content)
            .map_err(|e| DeclarchError::Other(format!("Failed to create module: {}", e)))?;

        Ok(())
    }

    /// Add package to KDL content string using AST-based manipulation
    ///
    /// Returns (updated_content, packages_added)
    fn add_package_to_content(
        &self,
        content: &str,
        package: &str,
        backend: Option<&str>,
    ) -> Result<(String, Vec<String>)> {
        use std::iter::Iterator;

        // Parse existing content to AST
        let mut doc: KdlDocument = content
            .parse()
            .map_err(|e| DeclarchError::Other(format!("KDL parsing error: {}", e)))?;

        // Detect default backend based on distro if not specified
        let backend_name = backend.unwrap_or_else(|| detect_default_backend());

        // Structure: pkg { backend { package } }
        // Step 1: Find or create 'pkg' node
        let pkg_node_idx = doc.nodes().iter().position(|n| n.name().value() == "pkg");

        let pkg_idx = if let Some(idx) = pkg_node_idx {
            idx
        } else {
            // Create pkg node
            let mut new_pkg = KdlNode::new("pkg");
            new_pkg.set_children(KdlDocument::new());
            doc.nodes_mut().push(new_pkg);
            doc.nodes().len() - 1
        };

        // Step 2: Find or create backend node inside pkg
        let pkg_node = &mut doc.nodes_mut()[pkg_idx];
        let backend_node_idx = if let Some(children) = pkg_node.children() {
            children
                .nodes()
                .iter()
                .position(|n| n.name().value() == backend_name)
        } else {
            None
        };

        if let Some(backend_idx) = backend_node_idx {
            // Backend node exists, check if package already exists
            if let Some(children) = pkg_node.children()
                && let Some(backend_node) = children.nodes().get(backend_idx)
                && let Some(backend_children) = backend_node.children()
            {
                for child in backend_children.nodes() {
                    if child.name().value() == package {
                        // Already exists, return unchanged
                        return Ok((content.to_string(), Vec::new()));
                    }
                }
            }

            // Add package to existing backend node
            if let Some(children) = pkg_node.children_mut() {
                let backend_node = &mut children.nodes_mut()[backend_idx];
                let package_node = KdlNode::new(package);

                if let Some(backend_children) = backend_node.children_mut() {
                    backend_children.nodes_mut().push(package_node);
                } else {
                    let mut backend_children_doc = KdlDocument::new();
                    backend_children_doc.nodes_mut().push(package_node);
                    backend_node.set_children(backend_children_doc);
                }
            }
        } else {
            // Create new backend node with package
            let mut backend_node = KdlNode::new(backend_name);
            let mut backend_children_doc = KdlDocument::new();
            backend_children_doc.nodes_mut().push(KdlNode::new(package));
            backend_node.set_children(backend_children_doc);

            if let Some(children) = pkg_node.children_mut() {
                children.nodes_mut().push(backend_node);
            } else {
                let mut children_doc = KdlDocument::new();
                children_doc.nodes_mut().push(backend_node);
                pkg_node.set_children(children_doc);
            }
        }

        // Generate KDL from modified AST
        let mut updated_content = doc.to_string();

        // Fix formatting issues from KDL library output
        // Problem 1: Add space before opening braces
        updated_content = updated_content.replace("pkg{", "pkg {");

        // Fix backend blocks - detect any word followed by newline that should be a block
        // Pattern: word\n or word\r\n followed by indented content or opening brace
        // This handles any backend name without hardcoding
        let backend_block_re = regex::Regex::new(r"(?m)^([a-zA-Z][a-zA-Z0-9_-]*)\s*\r?\n\s*\{")
            .map_err(|e| {
                crate::error::DeclarchError::ConfigError(format!("Invalid regex: {}", e))
            })?;

        updated_content = backend_block_re
            .replace_all(&updated_content, |caps: &regex::Captures| {
                format!("{} {{", &caps[1])
            })
            .to_string();

        // Problem 2: Add newlines between nodes (e.g., "}pkg" should be "}\npkg")
        // This happens when KDL library outputs multiple nodes without separation
        updated_content = updated_content
            .replace("}pkg", "}\npkg")
            .replace("}meta", "}\nmeta")
            .replace("}imports", "}\nimports")
            .replace("}hooks", "}\nhooks");

        // Problem 3: Add proper indentation for packages inside blocks
        let lines: Vec<&str> = updated_content.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut in_packages_block = false;

        for line in lines {
            let trimmed = line.trim();

            // Check if we're entering a pkg block
            if trimmed.starts_with("pkg") && trimmed.contains('{') {
                in_packages_block = true;
                // Preserve comments before the opening brace
                if line.contains("//") {
                    formatted_lines.push(line.to_string());
                } else {
                    formatted_lines.push(trimmed.to_string());
                }
                continue;
            }

            // Check if we're exiting a block
            if trimmed == "}" {
                in_packages_block = false;
                formatted_lines.push(trimmed.to_string());
                continue;
            }

            // Add indentation for package names inside packages blocks
            if in_packages_block && !trimmed.is_empty() {
                if trimmed.starts_with("//") {
                    // Keep comments as-is
                    formatted_lines.push(format!("  {}", trimmed));
                } else {
                    // Indent package names
                    formatted_lines.push(format!("  {}", trimmed));
                }
                continue;
            }

            // Keep other lines as-is
            if !trimmed.is_empty() {
                formatted_lines.push(trimmed.to_string());
            } else {
                formatted_lines.push(String::new());
            }
        }

        updated_content = formatted_lines.join("\n");

        Ok((updated_content, vec![package.to_string()]))
    }
}

/// Parse package string with optional backend
///
/// # Arguments
/// * `input` - Package string, e.g., "vim" or "soar:bat"
///
/// # Returns
/// * `(Option<backend>, package_name)` - Backend if specified, and package name
///
/// # Examples
/// ```
/// # use declarch::config::editor::parse_package_string;
/// # use declarch::error::Result;
/// # fn main() -> Result<()> {
/// assert_eq!(parse_package_string("vim")?, (None, "vim".to_string()));
/// assert_eq!(parse_package_string("soar:bat")?, (Some("soar".to_string()), "bat".to_string()));
/// assert_eq!(parse_package_string("npm:nodejs")?, (Some("npm".to_string()), "nodejs".to_string()));
/// # Ok(())
/// # }
/// ```
pub fn parse_package_string(input: &str) -> Result<(Option<String>, String)> {
    let trimmed = input.trim();

    // Check for empty string
    if trimmed.is_empty() {
        return Err(DeclarchError::Other(
            "Package name cannot be empty".to_string(),
        ));
    }

    // Check for multiple colons (e.g., "::package" or "backend::package")
    if trimmed.matches(':').count() > 1 {
        return Err(DeclarchError::Other(format!(
            "Invalid package format '{}'. Use 'backend:package' or 'package'",
            input
        )));
    }

    if let Some((backend, package)) = trimmed.split_once(':') {
        // Check for empty backend (":package")
        let backend = backend.trim();
        let package = package.trim();

        if backend.is_empty() {
            return Err(DeclarchError::Other(
                "Backend cannot be empty (use 'package' without colon)".to_string(),
            ));
        }

        // Check for empty package ("backend:")
        if package.is_empty() {
            return Err(DeclarchError::Other(
                "Package name cannot be empty (use 'backend:' without package)".to_string(),
            ));
        }

        // Validate backend name format (alphanumeric and hyphens only)
        if is_valid_backend(backend) {
            Ok((Some(backend.to_string()), package.to_string()))
        } else {
            Err(DeclarchError::Other(format!(
                "Invalid backend name: '{}'. Backend names must contain only letters, numbers, and hyphens",
                backend
            )))
        }
    } else {
        // No backend specified - validate package name is not empty
        if trimmed.is_empty() {
            return Err(DeclarchError::Other(
                "Package name cannot be empty".to_string(),
            ));
        }
        Ok((None, trimmed.to_string()))
    }
}

/// Check if a backend name is valid
/// Accepts any alphanumeric+hyphen name - no hardcoded backend list
fn is_valid_backend(backend: &str) -> bool {
    !backend.is_empty()
        && backend
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Create a backup of a KDL file before modification
pub fn backup_kdl_file(file_path: &Path) -> Result<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = file_path.with_extension(format!("kdl.bak.{}", timestamp));

    fs::copy(file_path, &backup_path).map_err(|e| {
        DeclarchError::Other(format!("Failed to backup {}: {}", file_path.display(), e))
    })?;

    Ok(backup_path)
}

/// Restore a KDL file from backup and clean up the backup file
pub fn restore_from_backup(backup_path: &Path) -> Result<()> {
    // Extract original path from backup path
    // Backup format: "filename.kdl.bak.TIMESTAMP.kdl" → "filename.kdl"
    let file_name = backup_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| DeclarchError::Other("Invalid backup path".to_string()))?;

    // Remove ".bak.TIMESTAMP.kdl" suffix to get original name
    // Pattern: "others.kdl.bak.20260129_204156.kdl" → "others.kdl"
    let original_name = file_name
        .split(".kdl.bak.")
        .next()
        .ok_or_else(|| DeclarchError::Other("Invalid backup filename format".to_string()))?
        .to_string()
        + ".kdl";

    let original_path = backup_path
        .parent()
        .ok_or_else(|| DeclarchError::Other("Cannot determine parent directory".to_string()))?
        .join(original_name);

    fs::copy(backup_path, &original_path).map_err(|e| {
        DeclarchError::Other(format!(
            "Failed to restore {}: {}",
            original_path.display(),
            e
        ))
    })?;

    // Clean up backup file
    let _ = fs::remove_file(backup_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_string_simple() {
        let (backend, package) = parse_package_string("vim").unwrap();
        assert!(backend.is_none());
        assert_eq!(package, "vim");
    }

    #[test]
    fn test_parse_package_string_with_backend() {
        let (backend, package) = parse_package_string("soar:bat").unwrap();
        assert_eq!(backend, Some("soar".to_string()));
        assert_eq!(package, "bat");
    }

    #[test]
    fn test_parse_package_string_npm() {
        let (backend, package) = parse_package_string("npm:node").unwrap();
        assert_eq!(backend, Some("npm".to_string()));
        assert_eq!(package, "node");
    }

    #[test]
    fn test_parse_package_string_invalid_backend_chars() {
        // Backend with invalid characters (space, special chars) should fail
        let result = parse_package_string("invalid backend:package");
        assert!(result.is_err());
        let result = parse_package_string("invalid@backend:package");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_string() {
        assert!(parse_package_string("").is_err());
    }

    #[test]
    fn test_validate_whitespace_only() {
        assert!(parse_package_string("   ").is_err());
    }

    #[test]
    fn test_validate_colon_only() {
        assert!(parse_package_string(":").is_err());
    }

    #[test]
    fn test_validate_multiple_colons() {
        assert!(parse_package_string("aur::bat").is_err());
        assert!(parse_package_string("::bat").is_err());
        assert!(parse_package_string("aur::").is_err());
    }

    #[test]
    fn test_validate_backend_empty() {
        assert!(parse_package_string(":bat").is_err());
    }

    #[test]
    fn test_validate_package_empty() {
        assert!(parse_package_string("aur:").is_err());
    }

    #[test]
    fn test_validate_trims_whitespace() {
        let (backend, package) = parse_package_string("  aur:bat  ").unwrap();
        assert_eq!(backend, Some("aur".to_string()));
        assert_eq!(package, "bat");
    }

    #[test]
    fn test_is_valid_backend() {
        // Valid backends: alphanumeric + hyphens
        assert!(is_valid_backend("aur"));
        assert!(is_valid_backend("soar"));
        assert!(is_valid_backend("flatpak"));
        assert!(is_valid_backend("npm"));
        assert!(is_valid_backend("my-custom-backend"));
        assert!(is_valid_backend("custom123"));

        // Invalid: empty or contains invalid characters
        assert!(!is_valid_backend(""));
        assert!(!is_valid_backend("invalid backend")); // space
        assert!(!is_valid_backend("invalid@backend")); // special char
        assert!(!is_valid_backend("invalid_backend")); // underscore
    }

    #[test]
    fn test_add_package_to_empty_content() {
        let editor = ConfigEditor::new();
        let content = "";
        let result = editor.add_package_to_content(content, "bat", Some("soar"));

        assert!(result.is_ok());
        let (updated, added) = result.unwrap();

        // Verify package was added
        assert_eq!(added, vec!["bat"]);
        assert!(updated.contains("pkg"));
        assert!(updated.contains("soar"));
        assert!(updated.contains("bat"));
        // Verify valid KDL syntax (has braces)
        assert!(updated.contains('{'));
        assert!(updated.contains('}'));
    }

    #[test]
    fn test_add_package_to_existing_block() {
        let editor = ConfigEditor::new();
        let content = "pkg {\n  soar {\n    vim\n  }\n}\n";
        let result = editor.add_package_to_content(content, "bat", Some("soar"));

        assert!(result.is_ok());
        let (updated, added) = result.unwrap();

        // Verify both packages exist
        assert_eq!(added, vec!["bat"]);
        assert!(updated.contains("vim"));
        assert!(updated.contains("bat"));
        // Verify it's still valid KDL with nested structure
        assert!(updated.contains("pkg"));
        assert!(updated.contains("soar"));
    }

    #[test]
    fn test_prevent_duplicates() {
        let editor = ConfigEditor::new();
        let content = "pkg {\n  soar {\n    bat\n  }\n}\n";
        let result = editor.add_package_to_content(content, "bat", Some("soar"));

        assert!(result.is_ok());
        let (updated, added) = result.unwrap();

        // Should return unchanged - no duplicate
        assert!(added.is_empty());
        assert_eq!(updated, content);
    }

    #[test]
    fn test_add_to_default_pkg_block() {
        let editor = ConfigEditor::new();
        let content = "pkg {\n  nano\n}\n";
        let result = editor.add_package_to_content(content, "vim", None);

        assert!(result.is_ok());
        let (updated, added) = result.unwrap();

        // Verify both packages exist in default block
        assert_eq!(added, vec!["vim"]);
        assert!(updated.contains("nano"));
        assert!(updated.contains("vim"));
        assert!(updated.contains("pkg {"));
    }

    #[test]
    fn test_add_multiple_backends() {
        let editor = ConfigEditor::new();
        let mut content = "";

        // Add AUR package (no specific backend = uses aur as default)
        let result = editor.add_package_to_content(content, "bat", None);
        assert!(result.is_ok());
        let (updated, _) = result.unwrap();
        content = &updated;

        // Add Soar package
        let result = editor.add_package_to_content(content, "fd", Some("soar"));
        assert!(result.is_ok());
        let (updated, _) = result.unwrap();

        // Verify nested structure: pkg { aur { bat } soar { fd } }
        assert!(updated.contains("pkg"));
        assert!(updated.contains("aur"));
        assert!(updated.contains("soar"));
        assert!(updated.contains("bat"));
        assert!(updated.contains("fd"));
    }
}
