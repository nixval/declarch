use super::SyncOptions;
use crate::config::loader;
use crate::error::Result;
use crate::project_identity;
use crate::ui as output;

pub(super) fn resolve_hooks_enabled(config: &loader::MergedConfig, options: &SyncOptions) -> bool {
    if !options.hooks {
        return false;
    }

    if config
        .policy
        .as_ref()
        .and_then(|p| p.forbid_hooks)
        .unwrap_or(false)
    {
        output::warning("Hooks are blocked by policy { forbid_hooks true }.");
        return false;
    }

    if config.is_experimental_enabled("enable-hooks") {
        return true;
    }

    output::warning(&format!(
        "Hooks were requested but blocked by policy. Add experimental {{ \"enable-hooks\" }} to {} to allow hook execution.",
        project_identity::CONFIG_FILE_BASENAME
    ));
    false
}

pub(super) fn enforce_sync_policy(config: &loader::MergedConfig) -> Result<()> {
    let Some(policy) = config.policy.as_ref() else {
        return Ok(());
    };

    if policy.require_backend.unwrap_or(false) {
        let legacy_default: Vec<_> = config
            .packages
            .keys()
            .filter(|pkg| pkg.backend.to_string() == "default")
            .map(|pkg| pkg.name.clone())
            .collect();

        if !legacy_default.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: require-backend=true but {} package(s) still use legacy default backend: {}",
                legacy_default.len(),
                legacy_default.join(", ")
            )));
        }
    }

    if policy.duplicate_is_error() {
        let duplicates = config.get_duplicates();
        if !duplicates.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: on-duplicate=error and {} duplicate declaration(s) were found",
                duplicates.len()
            )));
        }
    }

    if policy.conflict_is_error() {
        let conflicts = config.get_cross_backend_conflicts();
        if !conflicts.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: on-conflict=error and {} cross-backend conflict(s) were found",
                conflicts.len()
            )));
        }
    }

    Ok(())
}
