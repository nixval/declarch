use crate::config::loader;
use crate::ui as output;
use std::collections::HashMap;

fn parse_bool_option(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub(crate) fn apply_backend_option_overrides(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let Some(options) = config.backend_options.get(backend_name) else {
        return;
    };

    for (key, value) in options {
        let normalized = value.trim();
        let disable = normalized == "-";

        match key.as_str() {
            "noconfirm_flag" => {
                backend_config.noconfirm_flag = if disable { None } else { Some(value.clone()) }
            }
            "fallback" => {
                backend_config.fallback = if disable { None } else { Some(value.clone()) }
            }
            "install_cmd" => {
                if disable {
                    output::warning(&format!(
                        "Ignoring invalid disable sentinel for required option options:{} -> install_cmd=-",
                        backend_name
                    ));
                } else if !normalized.contains("{packages}") {
                    output::warning(&format!(
                        "Ignoring invalid install_cmd override for options:{}: missing '{{packages}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.install_cmd = value.clone();
                }
            }
            "remove_cmd" => {
                if disable {
                    backend_config.remove_cmd = None;
                } else if !normalized.contains("{packages}") {
                    output::warning(&format!(
                        "Ignoring invalid remove_cmd override for options:{}: missing '{{packages}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.remove_cmd = Some(value.clone());
                }
            }
            "list_cmd" => {
                backend_config.list_cmd = if disable { None } else { Some(value.clone()) }
            }
            "search_cmd" => {
                if disable {
                    backend_config.search_cmd = None;
                } else if !normalized.contains("{query}") {
                    output::warning(&format!(
                        "Ignoring invalid search_cmd override for options:{}: missing '{{query}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.search_cmd = Some(value.clone());
                }
            }
            "search_local_cmd" => {
                if disable {
                    backend_config.search_local_cmd = None;
                } else if !normalized.contains("{query}") {
                    output::warning(&format!(
                        "Ignoring invalid search_local_cmd override for options:{}: missing '{{query}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.search_local_cmd = Some(value.clone());
                }
            }
            "update_cmd" => {
                backend_config.update_cmd = if disable { None } else { Some(value.clone()) }
            }
            "cache_clean_cmd" => {
                backend_config.cache_clean_cmd = if disable { None } else { Some(value.clone()) }
            }
            "upgrade_cmd" => {
                backend_config.upgrade_cmd = if disable { None } else { Some(value.clone()) }
            }
            "needs_sudo" | "sudo" => {
                if let Some(parsed) = parse_bool_option(value) {
                    backend_config.needs_sudo = parsed;
                } else {
                    output::warning(&format!(
                        "Invalid boolean for options:{} -> {}={}",
                        backend_name, key, value
                    ));
                }
            }
            _ => {
                output::warning(&format!(
                    "Unknown backend option ignored: options:{} -> {}",
                    backend_name, key
                ));
            }
        }
    }
}

pub(crate) fn apply_backend_env_overrides(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let mut merged_env: HashMap<String, String> =
        backend_config.preinstall_env.clone().unwrap_or_default();

    for scope in ["global", backend_name] {
        if let Some(vars) = config.env.get(scope) {
            for var in vars {
                if let Some((k, v)) = var.split_once('=') {
                    merged_env.insert(k.trim().to_string(), v.trim().to_string());
                } else {
                    output::warning(&format!(
                        "Ignoring invalid env entry in env:{} -> {}",
                        scope, var
                    ));
                }
            }
        }
    }

    if merged_env.is_empty() {
        backend_config.preinstall_env = None;
    } else {
        backend_config.preinstall_env = Some(merged_env);
    }
}

pub(crate) fn apply_backend_package_sources(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let mut sources = backend_config
        .package_sources
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if let Some(repo_sources) = config.package_sources.get(backend_name) {
        for src in repo_sources {
            let trimmed = src.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !sources.iter().any(|s| s == trimmed) {
                sources.push(trimmed.to_string());
            }
        }
    }

    if sources.is_empty() {
        backend_config.package_sources = None;
    } else {
        backend_config.package_sources = Some(sources);
    }
}

#[cfg(test)]
mod tests;
