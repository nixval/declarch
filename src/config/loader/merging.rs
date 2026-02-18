use super::MergedConfig;
use crate::config::kdl::{McpConfig, RawConfig};
use crate::core::types::{Backend, PackageId};
use std::path::Path;

pub(super) struct PendingImports {
    pub(super) imports: Vec<String>,
    pub(super) backend_imports: Vec<String>,
}

pub(super) fn merge_raw_config(
    merged: &mut MergedConfig,
    raw: RawConfig,
    canonical_path: &Path,
) -> PendingImports {
    let RawConfig {
        imports,
        editor,
        packages_by_backend,
        excludes,
        project_metadata,
        conflicts,
        backend_options,
        env,
        package_sources,
        policy,
        lifecycle_actions,
        backend_imports,
        experimental,
        mcp,
    } = raw;

    for (backend_name, packages) in packages_by_backend {
        for pkg_entry in packages {
            let pkg_id = PackageId {
                name: pkg_entry.name,
                backend: Backend::from(backend_name.clone()),
            };
            merged
                .packages
                .entry(pkg_id)
                .or_default()
                .push(canonical_path.to_path_buf());
        }
    }

    merged.excludes.extend(excludes);

    if merged.project_metadata.is_none() {
        let has_description = project_metadata.description.is_some();
        let has_author = project_metadata.author.is_some();
        if has_description || has_author {
            merged.project_metadata = Some(project_metadata);
        }
    }

    if merged.editor.is_none() && editor.is_some() {
        merged.editor = editor;
    }

    merged.conflicts.extend(conflicts);

    for (backend, opts) in backend_options {
        merged
            .backend_options
            .entry(backend)
            .or_default()
            .extend(opts);
    }

    for (scope, vars) in env {
        merged.env.entry(scope).or_default().extend(vars);
    }

    for (backend, repos) in package_sources {
        merged
            .package_sources
            .entry(backend)
            .or_default()
            .extend(repos);
    }

    if policy.protected.iter().any(|p| !p.is_empty())
        || policy.orphans.is_some()
        || policy.require_backend.is_some()
        || policy.forbid_hooks.is_some()
        || policy.on_duplicate.is_some()
        || policy.on_conflict.is_some()
    {
        merged.policy = Some(policy);
    }

    if merged.lifecycle_actions.is_none() && !lifecycle_actions.actions.is_empty() {
        merged.lifecycle_actions = Some(lifecycle_actions);
    } else if let Some(ref mut merged_hooks) = merged.lifecycle_actions
        && !lifecycle_actions.actions.is_empty()
    {
        merged_hooks.actions.extend(lifecycle_actions.actions);
    }

    merged.experimental.extend(experimental);
    merge_mcp_policy(&mut merged.mcp, mcp);

    PendingImports {
        imports,
        backend_imports,
    }
}

fn merge_mcp_policy(target: &mut Option<McpConfig>, incoming: McpConfig) {
    if incoming.mode.is_none() && incoming.allow_tools.is_empty() {
        return;
    }

    let mcp = target.get_or_insert_with(McpConfig::default);
    if let Some(mode) = incoming.mode {
        mcp.mode = Some(mode);
    }

    for tool in incoming.allow_tools {
        if !mcp.allow_tools.iter().any(|t| t == &tool) {
            mcp.allow_tools.push(tool);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::merge_mcp_policy;
    use crate::config::kdl::McpConfig;

    #[test]
    fn merge_mcp_policy_deduplicates_allow_tools() {
        let mut merged = Some(McpConfig {
            mode: Some("read-only".to_string()),
            allow_tools: vec!["declarch_sync_apply".to_string()],
        });
        let incoming = McpConfig {
            mode: Some("write-enabled".to_string()),
            allow_tools: vec![
                "declarch_sync_apply".to_string(),
                "declarch_state_rm".to_string(),
            ],
        };

        merge_mcp_policy(&mut merged, incoming);

        let policy = merged.expect("policy should exist");
        assert_eq!(policy.mode.as_deref(), Some("write-enabled"));
        assert_eq!(policy.allow_tools.len(), 2);
        assert!(
            policy
                .allow_tools
                .iter()
                .any(|t| t == "declarch_sync_apply")
        );
        assert!(policy.allow_tools.iter().any(|t| t == "declarch_state_rm"));
    }
}
