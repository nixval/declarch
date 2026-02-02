//! KDL content parser
//!
//! Main parsing logic for KDL configuration files.

use crate::config::kdl_modules::types::{
    ActionType, Backend, ErrorBehavior, LifecycleAction, LifecyclePhase,
    PackageEntry, RawConfig,
};
use crate::config::kdl_modules::registry::BackendParserRegistry;
use crate::config::kdl_modules::helpers::{
    conflicts, env, hooks, meta, package_mappings, packages, policy, repositories,
};
use crate::error::Result;
use kdl::{KdlDocument, KdlNode};
use std::collections::HashMap;

/// Parse backend options: options:aur { noconfirm true }
pub fn parse_backend_options(
    node: &KdlNode,
    options: &mut HashMap<String, HashMap<String, String>>,
) -> Result<()> {
    // Check for colon syntax: options:aur
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified, apply to all? Or skip?
        // For now, skip if no backend specified
        return Ok(());
    };

    let mut opts = HashMap::new();

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
pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse().map_err(|e: kdl::KdlError| {
        // Provide more helpful error messages for common KDL syntax issues
        let err_msg = e.to_string();
        let hint = if err_msg.contains("unexpected token") {
            "\nHint: Check for missing quotes, unmatched brackets, or invalid characters."
        } else if err_msg.contains("unexpected end of file") {
            "\nHint: You might be missing a closing brace '}' or parenthesis."
        } else if err_msg.contains("expected") {
            "\nHint: Check that your KDL syntax follows the format: node-name \"value\" { ... }"
        } else {
            ""
        };

        crate::error::DeclarchError::ConfigError(format!("KDL parsing error: {}{}", err_msg, hint))
    })?;

    let mut config = RawConfig::new();

    let registry = BackendParserRegistry::new();

    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            "import" | "imports" => {
                packages::extract_strings(node, &mut config.imports);
                // DEBUG: Show what imports were parsed
                for _import in &config.imports {}
            }
            "exclude" | "excludes" => {
                packages::extract_mixed_values(node, &mut config.excludes);
            }
            "aliases-pkg" | "alias-pkg" => {
                package_mappings::extract_aliases(node, &mut config.package_mappings);
            }
            "description" => {
                // Parse description into project_metadata
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.project_metadata.description = Some(val.to_string());
                }
            }
            // NEW: Meta block
            "meta" => {
                meta::parse_meta_block(node, &mut config.project_metadata)?;
            }
            // NEW: Conflicts
            "conflicts" | "conflict" => {
                conflicts::parse_conflicts(node, &mut config.conflicts)?;
            }
            // NEW: Backend options
            name if name.starts_with("options") => {
                parse_backend_options(node, &mut config.backend_options)?;
            }
            // NEW: Environment variables
            name if name.starts_with("env") => {
                env::parse_env_vars(node, &mut config.env, None)?;
            }
            // NEW: Package repositories
            name if name.starts_with("repos") || name.starts_with("repositories") => {
                repositories::parse_repositories(node, &mut config.package_sources)?;
            }
            // NEW: Policy
            "policy" => {
                policy::parse_policy(node, &mut config.policy)?;
            }
            // NEW: Hooks
            "hooks" => {
                hooks::parse_hooks(node, &mut config.lifecycle_actions)?;
            }
            // NEW: Simplified flat hooks (backward compatibility)
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
            // Parse packages with flexible syntax using the registry
            name if name.starts_with("packages") => {
                registry.parse_packages_node(node, &mut config)?;
            }
            // Legacy syntax support (with deprecation warning in the future)
            "aur-packages" | "aur-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .packages_for_mut(&Backend::Aur)
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "soar-packages" | "soar-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .packages_for_mut(&Backend::Soar)
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "flatpak-packages" | "flatpak-package" => {
                let packages = packages::extract_mixed_values_return(node);
                config
                    .packages_for_mut(&Backend::Flatpak)
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            _ => {}
        }
    }

    Ok(config)
}
