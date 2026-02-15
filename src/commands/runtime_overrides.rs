use crate::backends::config::BackendConfig;
use crate::config::loader;
use crate::ui as output;
use crate::utils::paths;

/// Load merged runtime config for backend override application.
/// Returns default config if no root config exists or if parsing fails.
pub(crate) fn load_runtime_config_for_command(command_name: &str) -> loader::MergedConfig {
    match paths::config_file() {
        Ok(path) if path.exists() => match loader::load_root_config(&path) {
            Ok(cfg) => cfg,
            Err(e) => {
                output::warning(&format!(
                    "Failed to load config overrides for {}: {}",
                    command_name, e
                ));
                loader::MergedConfig::default()
            }
        },
        _ => loader::MergedConfig::default(),
    }
}

/// Apply all supported runtime backend overrides consistently.
pub(crate) fn apply_runtime_backend_overrides(
    backend_config: &mut BackendConfig,
    backend_name: &str,
    runtime_config: &loader::MergedConfig,
) {
    crate::commands::sync::apply_backend_option_overrides(
        backend_config,
        backend_name,
        runtime_config,
    );
    crate::commands::sync::apply_backend_env_overrides(
        backend_config,
        backend_name,
        runtime_config,
    );
    crate::commands::sync::apply_backend_package_sources(
        backend_config,
        backend_name,
        runtime_config,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn apply_runtime_backend_overrides_applies_options_env_and_sources() {
        let mut backend = BackendConfig {
            name: "paru".to_string(),
            install_cmd: "paru -S {packages}".to_string(),
            noconfirm_flag: Some("--noconfirm".to_string()),
            ..Default::default()
        };

        let mut merged = loader::MergedConfig::default();
        let mut options = HashMap::new();
        options.insert("noconfirm_flag".to_string(), "--yes".to_string());
        options.insert("needs_sudo".to_string(), "true".to_string());
        merged.backend_options.insert("paru".to_string(), options);
        merged.env.insert(
            "paru".to_string(),
            vec!["A=1".to_string(), "B=2".to_string()],
        );
        merged.package_sources.insert(
            "paru".to_string(),
            vec!["core".to_string(), "extra".to_string()],
        );

        apply_runtime_backend_overrides(&mut backend, "paru", &merged);

        assert_eq!(backend.noconfirm_flag.as_deref(), Some("--yes"));
        assert!(backend.needs_sudo);
        let env = backend.preinstall_env.expect("env set");
        assert_eq!(env.get("A").map(String::as_str), Some("1"));
        assert_eq!(env.get("B").map(String::as_str), Some("2"));
        assert_eq!(
            backend.package_sources,
            Some(vec!["core".to_string(), "extra".to_string()])
        );
    }
}
