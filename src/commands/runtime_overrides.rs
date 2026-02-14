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
