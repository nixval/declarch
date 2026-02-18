use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::error::Result;
use crate::packages::traits::PackageManager;
use crate::project_identity;
use crate::ui as output;

pub(super) fn check_backends_dynamically(verbose: bool) -> Result<Vec<String>> {
    let mut available = Vec::new();
    let runtime_config = load_runtime_config_for_command("doctor backend checks");

    match crate::backends::load_all_backends_unified() {
        Ok(backends) => {
            for (name, mut config) in backends {
                apply_runtime_backend_overrides(&mut config, &name, &runtime_config);

                if !crate::utils::platform::backend_supports_current_os(&config) {
                    if verbose {
                        output::info(&format!("{}: Skipped (not for this OS)", name));
                    }
                    continue;
                }

                let manager = crate::backends::GenericManager::from_config(
                    config,
                    crate::core::types::Backend::from(name.as_str()),
                    false,
                );

                if manager.is_available() {
                    if verbose {
                        output::success(&format!("{}: Available", name));
                    }
                    available.push(name);
                } else {
                    output::warning(&format!("{}: Backend binary not found", name));
                }
            }
        }
        Err(e) => {
            output::warning(&format!("Could not load backend configs: {}", e));
        }
    }

    if available.is_empty() {
        output::warning("No backends configured or available");
        output::info(&format!(
            "Run '{}' to add a backend",
            project_identity::cli_with("init --backend <name>")
        ));
    }

    Ok(available)
}

pub(crate) fn collect_state_signature_duplicates(
    state: &crate::state::types::State,
) -> Vec<(String, Vec<String>)> {
    let mut signatures: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (key, pkg_state) in &state.packages {
        let signature = format!("{}:{}", pkg_state.backend, pkg_state.config_name);
        signatures.entry(signature).or_default().push(key.clone());
    }

    let mut duplicates: Vec<(String, Vec<String>)> = signatures
        .into_iter()
        .filter_map(|(signature, keys)| (keys.len() > 1).then_some((signature, keys)))
        .collect();
    duplicates.sort_by(|a, b| a.0.cmp(&b.0));
    duplicates
}
