use crate::config::kdl_modules::helpers::{
    conflicts, env, hooks, meta, packages, policy, repositories,
};
use crate::config::kdl_modules::types::{
    ActionType, ErrorBehavior, LifecycleAction, LifecyclePhase, McpConfig, PackageEntry, RawConfig,
};
use crate::error::Result;
use kdl::KdlNode;
use std::collections::HashSet;

pub(super) fn map_node_into_config(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    let node_name = node.name().value();

    match node_name {
        "import" | "imports" => {
            packages::extract_strings(node, &mut config.imports);
        }
        "exclude" | "excludes" => {
            packages::extract_mixed_values(node, &mut config.excludes);
        }
        "backends" => {
            packages::extract_strings(node, &mut config.backend_imports);
        }
        "experimental" => {
            parse_experimental_flags(node, &mut config.experimental);
        }
        "mcp" => {
            parse_mcp_policy(node, &mut config.mcp);
        }

        "description" => {
            if let Some(entry) = node.entries().first()
                && let Some(val) = entry.value().as_string()
            {
                config.project_metadata.description = Some(val.to_string());
            }
        }
        "editor" => {
            if let Some(entry) = node.entries().first()
                && let Some(val) = entry.value().as_string()
            {
                config.editor = Some(val.to_string());
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
        name if name == "pkg" || name.starts_with("pkg:") => {
            parse_pkg_node(node, config)?;
        }
        name if name == "packages" || name.starts_with("packages:") => {
            parse_packages_node_legacy(node, config)?;
        }
        "backend" => {}
        _ => {}
    }

    Ok(())
}

pub(super) fn parse_backend_options(
    node: &KdlNode,
    options: &mut std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) -> Result<()> {
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        return Ok(());
    };

    let mut opts = std::collections::HashMap::new();

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
                opts.insert(key.to_string(), "true".to_string());
            }
        }
    }

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

fn parse_experimental_flags(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let mut pushed_from_entries = false;
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                    pushed_from_entries = true;
                }
            }

            if !pushed_from_entries {
                let name = child.name().value();
                if !name.is_empty() && !name.starts_with("//") {
                    target.push(name.to_string());
                }
            }
        }
    }
}

fn parse_mcp_policy(node: &KdlNode, target: &mut McpConfig) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            let lowered = val.trim().to_lowercase();
            if lowered == "read-only" || lowered == "write-enabled" {
                target.mode = Some(lowered);
            }
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            match name {
                "mode" => {
                    if let Some(val) = child.entries().first().and_then(|e| e.value().as_string()) {
                        target.mode = Some(val.trim().to_lowercase());
                    }
                }
                "allow-tools" | "allow_tools" => {
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            target.allow_tools.push(val.to_string());
                        }
                    }
                    if let Some(grand_children) = child.children() {
                        for grand_child in grand_children.nodes() {
                            for entry in grand_child.entries() {
                                if let Some(val) = entry.value().as_string() {
                                    target.allow_tools.push(val.to_string());
                                }
                            }
                            let child_name = grand_child.name().value();
                            if !child_name.is_empty() && !child_name.starts_with("//") {
                                target.allow_tools.push(child_name.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut seen = HashSet::new();
    target.allow_tools.retain(|t| seen.insert(t.clone()));
}

fn parse_pkg_node(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    let node_name = node.name().value();
    if let Some((_, backend)) = node_name.split_once(':') {
        let packages = extract_packages_from_node(node);
        if !packages.is_empty() {
            config
                .packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .extend(packages);
        }
        return Ok(());
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            if let Some((backend, package)) = child_name.split_once(':') {
                config
                    .packages_by_backend
                    .entry(backend.to_string())
                    .or_default()
                    .push(PackageEntry {
                        name: package.to_string(),
                    });
            } else {
                let packages = extract_packages_from_node(child);
                if !packages.is_empty() {
                    config
                        .packages_by_backend
                        .entry(child_name.to_string())
                        .or_default()
                        .extend(packages);
                }
            }
        }
    }

    for entry in node.entries() {
        if let Some(val) = entry.value().as_string()
            && let Some((backend, package)) = val.split_once(':')
        {
            config
                .packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .push(PackageEntry {
                    name: package.to_string(),
                });
        }
    }

    Ok(())
}

fn extract_packages_from_node(node: &KdlNode) -> Vec<PackageEntry> {
    let mut packages = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            packages.push(PackageEntry {
                name: child.name().value().to_string(),
            });

            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    packages.push(PackageEntry {
                        name: val.to_string(),
                    });
                }
            }
        }
    }

    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            packages.push(PackageEntry {
                name: val.to_string(),
            });
        }
    }

    packages
}

fn parse_packages_node_legacy(node: &KdlNode, config: &mut RawConfig) -> Result<()> {
    let node_name = node.name().value();

    if let Some((_, backend)) = node_name.split_once(':') {
        let packages = extract_packages_from_node(node);
        if !packages.is_empty() {
            config
                .packages_by_backend
                .entry(backend.to_string())
                .or_default()
                .extend(packages);
        }
        return Ok(());
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            if child_name.contains(':') {
                if let Some((backend, package)) = child_name.split_once(':') {
                    config
                        .packages_by_backend
                        .entry(backend.to_string())
                        .or_default()
                        .push(PackageEntry {
                            name: package.to_string(),
                        });
                }
            } else if child.children().is_some() {
                let packages = extract_packages_from_node(child);
                if !packages.is_empty() {
                    config
                        .packages_by_backend
                        .entry(child_name.to_string())
                        .or_default()
                        .extend(packages);
                }
            } else {
                config
                    .packages_by_backend
                    .entry("default".to_string())
                    .or_default()
                    .push(PackageEntry {
                        name: child_name.to_string(),
                    });

                for entry in child.entries() {
                    if let Some(val) = entry.value().as_string() {
                        config
                            .packages_by_backend
                            .entry("default".to_string())
                            .or_default()
                            .push(PackageEntry {
                                name: val.to_string(),
                            });
                    }
                }
            }
        }
    }

    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            if val.contains(':') {
                if let Some((backend, package)) = val.split_once(':') {
                    config
                        .packages_by_backend
                        .entry(backend.to_string())
                        .or_default()
                        .push(PackageEntry {
                            name: package.to_string(),
                        });
                }
            } else {
                config
                    .packages_by_backend
                    .entry("default".to_string())
                    .or_default()
                    .push(PackageEntry {
                        name: val.to_string(),
                    });
            }
        }
    }

    Ok(())
}
