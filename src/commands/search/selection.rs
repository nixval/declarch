use super::{SearchOptions, canonical_backend_group};
use crate::core::types::Backend;
use crate::error::Result;
use crate::project_identity;
use crate::ui as output;
use std::collections::{HashMap, HashSet};

pub(super) fn get_backends_to_search(
    options: &SearchOptions,
    backend_configs: &HashMap<String, crate::backends::config::BackendConfig>,
    machine_mode: bool,
) -> Result<(Vec<Backend>, Vec<String>)> {
    let (result, unknown, unsupported, os_mismatch) =
        select_backends_to_search(backend_configs, options.backends.as_ref(), options.local);
    let mut warnings = Vec::new();

    if !unknown.is_empty() {
        let msg = format!("Unknown backend(s): {}", unknown.join(", "));
        if machine_mode {
            warnings.push(msg);
        } else {
            output::warning(&msg);
        }
    }
    if !unsupported.is_empty() && should_emit_selection_warning(options) {
        let capability = if options.local {
            "local search/list support"
        } else {
            "search support"
        };
        let msg = format!(
            "Skipped backend(s) without {}: {}",
            capability,
            unsupported.join(", ")
        );
        if machine_mode {
            warnings.push(msg);
        } else {
            output::warning(&msg);
        }
    }
    if !os_mismatch.is_empty() && should_emit_selection_warning(options) {
        let msg = format!(
            "Skipped backend(s) that are not for this OS: {}",
            os_mismatch.join(", ")
        );
        if machine_mode {
            warnings.push(msg);
            warnings.push(
                "This is normal when one config is shared across different machines.".to_string(),
            );
        } else {
            output::warning(&msg);
            output::info("This is normal when one config is shared across different machines.");
        }
    }

    if result.is_empty() {
        let msg = if options.local {
            "No backends with local search/list support configured".to_string()
        } else {
            "No backends with search support configured".to_string()
        };
        if options.local {
            if machine_mode {
                warnings.push(msg);
            } else {
                output::warning("No backends with local search/list support configured");
            }
        } else if machine_mode {
            warnings.push(msg);
        } else {
            output::warning("No backends with search support configured");
        }
        if machine_mode {
            warnings.push(format!(
                "Run '{}' to add a backend",
                project_identity::cli_with("init --backend <name>")
            ));
        } else {
            output::info(&format!(
                "Run '{}' to add a backend",
                project_identity::cli_with("init --backend <name>")
            ));
        }
    }

    Ok((result, warnings))
}

pub(super) fn should_emit_selection_warning(options: &SearchOptions) -> bool {
    // Keep default output focused: show capability/OS skip warnings when user
    // explicitly requested backends, or when verbose diagnostics are enabled.
    options.verbose || options.backends.as_ref().is_some_and(|b| !b.is_empty())
}

pub(super) fn select_backends_to_search(
    all_backends: &HashMap<String, crate::backends::config::BackendConfig>,
    requested_backends: Option<&Vec<String>>,
    local_mode: bool,
) -> (Vec<Backend>, Vec<String>, Vec<String>, Vec<String>) {
    let supports_mode = |config: &crate::backends::config::BackendConfig| {
        if local_mode {
            config.search_local_cmd.is_some() || config.list_cmd.is_some()
        } else {
            config.search_cmd.is_some()
        }
    };

    let mut selected = Vec::new();
    let mut unknown = Vec::new();
    let mut unsupported = Vec::new();
    let mut os_mismatch = Vec::new();

    if let Some(requested) = requested_backends {
        for name in requested {
            match all_backends.get(name) {
                Some(config)
                    if crate::utils::platform::backend_supports_current_os(config)
                        && supports_mode(config) =>
                {
                    selected.push(Backend::from(name.as_str()))
                }
                Some(config) if !crate::utils::platform::backend_supports_current_os(config) => {
                    os_mismatch.push(name.clone())
                }
                Some(_) => unsupported.push(name.clone()),
                None => unknown.push(name.clone()),
            }
        }
    } else {
        let mut entries: Vec<_> = all_backends.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut selected_local_groups: HashSet<String> = HashSet::new();
        for (name, config) in entries {
            if crate::utils::platform::backend_supports_current_os(config) && supports_mode(config)
            {
                if local_mode {
                    let group = canonical_backend_group(name).to_string();
                    if selected_local_groups.contains(&group) {
                        continue;
                    }
                    selected_local_groups.insert(group);
                }
                selected.push(Backend::from(name.as_str()));
            }
        }
    }

    unknown.sort();
    unsupported.sort();
    os_mismatch.sort();

    (selected, unknown, unsupported, os_mismatch)
}
