//! Config Editor Module
//!
//! This module provides functionality to programmatically edit KDL configuration files.
//! It's used by the `install` command to add packages to config files.

mod backup_ops;
mod default_backend;
mod formatting;
mod package_spec;
mod path_ops;

use crate::error::{DeclarchError, Result};
pub use backup_ops::{backup_kdl_file, restore_from_backup};
use default_backend::detect_default_backend;
use kdl::{KdlDocument, KdlNode};
#[cfg(test)]
use package_spec::is_valid_backend;
pub use package_spec::parse_package_string;
use path_ops::{create_default_module, resolve_module_path};
use std::fs;
use std::path::PathBuf;

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
        let target_file = resolve_module_path(module)?;

        // Create backup if file exists
        let backup_path = if target_file.exists() {
            Some(backup_kdl_file(&target_file)?)
        } else {
            None
        };

        // Load or create file
        let created_new_file = if !target_file.exists() {
            create_default_module(&target_file)?;
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

        // Generate KDL from modified AST and normalize formatting.
        let updated_content = formatting::normalize_kdl_output(doc.to_string())?;

        Ok((updated_content, vec![package.to_string()]))
    }
}

#[cfg(test)]
mod tests;
