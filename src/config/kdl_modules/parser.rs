//! KDL content parser
//!
//! Main parsing logic for KDL configuration files.
//! 
//! In v0.6+, this uses a fully generic approach where all packages are
//! stored in unified storage (packages_by_backend).

use crate::config::kdl_modules::types::{
    ActionType, ErrorBehavior, LifecycleAction, LifecyclePhase,
    PackageEntry, RawConfig,
};
use crate::config::kdl_modules::helpers::{
    conflicts, env, hooks, meta, package_mappings, packages, policy, repositories,
};
use crate::error::Result;
use kdl::{KdlDocument, KdlNode};

/// Parse backend options: options:backend_name { noconfirm true }
pub fn parse_backend_options(
    node: &KdlNode,
    options: &mut std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) -> Result<()> {
    // Check for colon syntax: options:paru
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified - skip
        return Ok(());
    };

    let mut opts = std::collections::HashMap::new();

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            if let Some(val) = meta::get_first_string(child) {
                opts.insert(key.to_string(), val);
            } else if let Some(val) = child.entries().first()
                && let Some(val) = val.value().as_string()
            {
                opts.insert(key.to_string(), val.to_string());
            } else {
                // Boolean flag without value
                opts.insert(key.to_string(), "true".to_string());
            }
        }
    }

    // Extract from string arguments (key=value format)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string()
            && let Some((key, v)) = val.split_once('=')
        {
            opts.insert(key.to_string(), v.to_string());
        }
    }

    if !opts.is_empty() {
        options.insert(backend_name, opts);
    }

    Ok(())
}

/// Parse KDL content into RawConfig
///
/// This is the main entry point for parsing KDL configuration files.
/// Uses fully unified package storage - no backend-specific fields.
pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    parse_kdl_content_with_path(content, None)
}

/// Parse KDL content with file path for better error reporting
///
/// Shows detailed error messages with line numbers, visual indicators,
/// and helpful hints similar to Rust compiler errors.
pub fn parse_kdl_content_with_path(content: &str, file_path: Option<&str>) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse().map_err(|e: kdl::KdlError| {
        let report = super::error_reporter::format_error_report(content, file_path, &e);
        crate::error::DeclarchError::ConfigError(report)
    })?;

    let mut config = RawConfig::default();

    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            "import" | "imports" => {
                packages::extract_strings(node, &mut config.imports);
            }
            "exclude" | "excludes" => {
                packages::extract_mixed_values(node, &mut config.excludes);
            }
            "aliases-pkg" | "alias-pkg" => {
                package_mappings::extract_aliases(node, &mut config.package_mappings);
            }
            "description" => {
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.project_metadata.description = Some(val.to_string());
                }
            }
            "meta" => {
                meta::parse_meta_block(node, &mut config.project_metadata)?;
            }
            "conflicts" | "conflict" => {
                conflicts::parse_conflicts(node, &mut config.conflicts)?;
            }
            name if name.starts_with("options") => {
                parse_backend_options(node, &mut config.backend_options)?;
            }
            name if name.starts_with("env") => {
                env::parse_env_vars(node, &mut config.env, None)?;
            }
            name if name.starts_with("repos") || name.starts_with("repositories") => {
                repositories::parse_repositories(node, &mut config.package_sources)?;
            }
            "policy" => {
                policy::parse_policy(node, &mut config.policy)?;
            }
            "hooks" => {
                hooks::parse_hooks(node, &mut config.lifecycle_actions)?;
            }
            "on-sync" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.lifecycle_actions.actions.push(LifecycleAction {
                        command: val,
                        action_type: ActionType::User,
                        phase: LifecyclePhase::PostSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            "on-sync-sudo" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.lifecycle_actions.actions.push(LifecycleAction {
                        command: val,
                        action_type: ActionType::Root,
                        phase: LifecyclePhase::PostSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            "on-pre-sync" => {
                if let Some(val) = meta::get_first_string(node) {
                    config.lifecycle_actions.actions.push(LifecycleAction {
                        command: val,
                        action_type: ActionType::User,
                        phase: LifecyclePhase::PreSync,
                        package: None,
                        conditions: vec![],
                        error_behavior: ErrorBehavior::default(),
                    });
                }
            }
            // Unified pkg syntax: pkg { backend { packages } } or pkg:backend { ... }
            name if name == "pkg" || name.starts_with("pkg:") => {
                parse_pkg_node(node, &mut config)?;
            }
            // Legacy packages syntax (deprecated but still supported for migration)
            name if name == "packages" || name.starts_with("packages:") => {
                parse_packages_node_legacy(node, &mut config)?;
            }
            // Backend definitions - these are parsed separately by the backend registry
            // Ignore them here since backends.kdl may be imported as a module
            "backend" => {}
            _ => {}
        }
    }

    Ok(config)
}

/// Parse unified pkg node: pkg { backend { packages } }
fn parse_pkg_node(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    // Handle pkg:backend { packages } syntax
    let node_name = node.name().value();
    if let Some((_, backend)) = node_name.split_once(':') {
        // Direct backend specification: pkg:paru { ... }
        let packages = extract_packages_from_node(node)?;
        if !packages.is_empty() {
            config.packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .extend(packages);
        }
        return Ok(());
    }

    // Handle nested blocks: pkg { paru { ... } npm { ... } }
    // Also handles inline prefix: pkg { paru:hyprland npm:typescript }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            
            // Check if child name contains colon (inline prefix syntax)
            if let Some((backend, package)) = child_name.split_once(':') {
                // Inline prefix: paru:hyprland
                config.packages_by_backend
                    .entry(backend.to_string())
                    .or_default()
                    .push(PackageEntry { name: package.to_string() });
            } else {
                // Regular backend block: paru { hyprland waybar }
                let packages = extract_packages_from_node(child)?;
                if !packages.is_empty() {
                    config.packages_by_backend
                        .entry(child_name.to_string())
                        .or_default()
                        .extend(packages);
                }
            }
        }
    }

    // Handle inline entries: pkg { "paru:package1" "npm:package2" }
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string()
            && let Some((backend, package)) = val.split_once(':') {
                config.packages_by_backend
                    .entry(backend.to_string())
                    .or_default()
                    .push(PackageEntry { name: package.to_string() });
            }
    }

    Ok(())
}

/// Extract packages from a node (children and entries)
fn extract_packages_from_node(node: &KdlNode) -> Result<Vec<PackageEntry>> {
    let mut packages = Vec::new();

    // Extract from children (nested package names)
    if let Some(children) = node.children() {
        for child in children.nodes() {
            packages.push(PackageEntry { 
                name: child.name().value().to_string() 
            });
            
            // Also check for string arguments
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    packages.push(PackageEntry { name: val.to_string() });
                }
            }
        }
    }

    // Extract from direct string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            packages.push(PackageEntry { name: val.to_string() });
        }
    }

    Ok(packages)
}

/// Parse legacy packages node (deprecated but supported for migration)
/// Handles: packages { pkg1 pkg2 }, packages:paru { ... }, packages { paru { ... } }
fn parse_packages_node_legacy(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    let node_name = node.name().value();

    // Check for colon syntax: packages:backend
    if let Some((_, backend)) = node_name.split_once(':') {
        let packages = extract_packages_from_node(node)?;
        if !packages.is_empty() {
            config.packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .extend(packages);
        }
        return Ok(());
    }

    // Check for nested children (backend blocks or package names)
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            
            // Check if child name contains colon (inline prefix)
            if child_name.contains(':') {
                if let Some((backend, package)) = child_name.split_once(':') {
                    config.packages_by_backend
                        .entry(backend.to_string())
                        .or_default()
                        .push(PackageEntry { name: package.to_string() });
                }
            } else if child.children().is_some() {
                // Child has nested structure - treat as backend block
                // e.g., packages { paru { hyprland } }
                let packages = extract_packages_from_node(child)?;
                if !packages.is_empty() {
                    config.packages_by_backend
                        .entry(child_name.to_string())
                        .or_default()
                        .extend(packages);
                }
            } else {
                // Child is a simple name - treat as package in "default"
                // e.g., packages { hyprland waybar }
                config.packages_by_backend
                    .entry("default".to_string())
                    .or_default()
                    .push(PackageEntry { name: child_name.to_string() });
                
                // Also check for string arguments in the child node
                for entry in child.entries() {
                    if let Some(val) = entry.value().as_string() {
                        config.packages_by_backend
                            .entry("default".to_string())
                            .or_default()
                            .push(PackageEntry { name: val.to_string() });
                    }
                }
            }
        }
    }

    // Extract direct entries
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            if val.contains(':') {
                if let Some((backend, package)) = val.split_once(':') {
                    config.packages_by_backend
                        .entry(backend.to_string())
                        .or_default()
                        .push(PackageEntry { name: package.to_string() });
                }
            } else {
                config.packages_by_backend
                    .entry("default".to_string())
                    .or_default()
                    .push(PackageEntry { name: val.to_string() });
            }
        }
    }

    Ok(())
}
