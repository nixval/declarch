use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::config::loader;
use crate::error::Result;
use crate::packages::traits::PackageManager;
use crate::project_identity;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use crate::utils::update_check::{InstallOwner, is_managed_by_package_manager, update_hint_cached};

mod output_view;

pub struct InfoOptions {
    pub doctor: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
    pub backend: Option<String>,
    pub package: Option<String>,
    pub verbose: bool,
}

pub fn run(options: InfoOptions) -> Result<()> {
    if options.doctor {
        return run_doctor(options.verbose);
    }

    let result = run_info(&options);
    if result.is_ok() && !matches!(options.format.as_deref(), Some("json" | "yaml")) {
        maybe_print_update_notification();
    }
    result
}

fn run_info(options: &InfoOptions) -> Result<()> {
    let state = state::io::load_state()?;

    let filtered_packages: Vec<(&String, &state::types::PackageState)> =
        if options.backend.is_some() || options.package.is_some() {
            let backend_filter = options.backend.as_deref();
            let package_filter = options.package.as_deref();

            state
                .packages
                .iter()
                .filter(|(key, pkg_state)| {
                    let name = extract_package_name(key);

                    if let Some(filter_pkg) = package_filter
                        && !name.contains(filter_pkg)
                    {
                        return false;
                    }

                    if let Some(filter_backend) = backend_filter {
                        pkg_state.backend == crate::core::types::Backend::from(filter_backend)
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            state.packages.iter().collect()
        };

    let format_str = options.format.as_deref().unwrap_or("table");
    if options.verbose && !matches!(format_str, "json" | "yaml") {
        output::header("Info Context");
        output::keyval(
            "Backend filter",
            options.backend.as_deref().unwrap_or("(none)"),
        );
        output::keyval(
            "Package filter",
            options.package.as_deref().unwrap_or("(none)"),
        );
        output::keyval(
            "State file",
            &state::io::get_state_path()?.display().to_string(),
        );
    }

    match format_str {
        "json" => output_json_filtered(&filtered_packages, options.output_version.as_deref()),
        "yaml" => output_yaml_filtered(&filtered_packages, options.output_version.as_deref()),
        _ => output_table_filtered(&state, &filtered_packages),
    }
}

fn output_table_filtered(
    state: &state::types::State,
    filtered_packages: &[(&String, &state::types::PackageState)],
) -> Result<()> {
    output_view::output_table_filtered(state, filtered_packages)
}

fn output_json_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    output_view::output_json_filtered(filtered_packages, output_version)
}

fn output_yaml_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    output_view::output_yaml_filtered(filtered_packages, output_version)
}

fn extract_package_name(key: &str) -> &str {
    key.split_once(':').map(|(_, n)| n).unwrap_or(key)
}

fn run_doctor(verbose: bool) -> Result<()> {
    output::header("System Diagnosis");
    let mut all_ok = true;
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    output::info(&format!("OS/Arch: {}/{}", os, arch));
    if verbose {
        output::info("Runtime paths");
        output::indent(
            &format!("• Config dir: {}", paths::config_dir()?.display()),
            2,
        );
        output::indent(
            &format!("• Config file: {}", paths::config_file()?.display()),
            2,
        );
        output::indent(
            &format!("• State file: {}", state::io::get_state_path()?.display()),
            2,
        );
    } else {
        output::info("Tip: run with --verbose to show runtime paths.");
    }
    if os == "macos" || os == "windows" {
        output::warning(
            "This OS path is experimental (alpha). Core workflow should work, but expect rough edges.",
        );
    }
    output::separator();

    // Check 1: Config file
    output::info("Checking configuration file...");
    let config_path = paths::config_file()?;
    if config_path.exists() {
        if verbose {
            output::success(&format!("Config found: {}", config_path.display()));
        } else {
            output::success("Config found");
        }

        match loader::load_root_config(&config_path) {
            Ok(config) => {
                output::success(&format!(
                    "Config valid: {} packages defined",
                    config.packages.len()
                ));
            }
            Err(e) => {
                output::error(&format!("Config parse error: {}", e));
                all_ok = false;
            }
        }
    } else {
        if verbose {
            output::warning(&format!("Config not found: {}", config_path.display()));
        } else {
            output::warning("Config not found");
        }
        output::info(&format!(
            "Run '{}' to create a configuration",
            project_identity::cli_with("init")
        ));
        all_ok = false;
    }

    // Check 2: State file
    output::info("Checking state file...");
    let state_path = state::io::get_state_path()?;
    if state_path.exists() {
        if verbose {
            output::success(&format!("State found: {}", state_path.display()));
        } else {
            output::success("State found");
        }

        match state::io::load_state() {
            Ok(state) => {
                output::success(&format!(
                    "State valid: {} packages tracked",
                    state.packages.len()
                ));

                if config_path.exists()
                    && let Ok(config) = loader::load_root_config(&config_path)
                {
                    use crate::core::types::PackageId;
                    use std::collections::HashSet;

                    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
                    let mut orphan_count = 0;

                    for pkg_state in state.packages.values() {
                        let pkg_id = PackageId {
                            backend: pkg_state.backend.clone(),
                            name: pkg_state.config_name.clone(),
                        };
                        if !config_set.contains(&pkg_id) {
                            orphan_count += 1;
                        }
                    }

                    if orphan_count > 0 {
                        output::warning(&format!(
                            "Found {} orphan packages (not in config)",
                            orphan_count
                        ));
                        output::info(&format!(
                            "Run '{}' to see them",
                            project_identity::cli_with("info --list --scope orphans")
                        ));
                        output::info(&format!(
                            "Run '{}' to remove orphans",
                            project_identity::cli_with("sync prune")
                        ));
                    } else {
                        output::success("No orphan packages found");
                    }
                }
            }
            Err(e) => {
                output::error(&format!("State load error: {}", e));
                all_ok = false;
            }
        }
    } else {
        output::warning("State not found");
        output::info(&format!(
            "Run '{}' to create initial state",
            project_identity::cli_with("sync")
        ));
    }

    // Check 3: Backend availability (dynamic)
    output::info("Checking backends...");
    let available_backends = check_backends_dynamically(verbose)?;

    // Check 4: State consistency
    output::info("Checking state consistency...");
    let Some(state) = state_path
        .exists()
        .then(state::io::load_state)
        .transpose()?
    else {
        return Ok(());
    };

    let duplicate_signatures = collect_state_signature_duplicates(&state);
    if !duplicate_signatures.is_empty() {
        output::warning(&format!(
            "State has {} duplicate package signature(s) (backend:name)",
            duplicate_signatures.len()
        ));
        if verbose {
            for (signature, keys) in duplicate_signatures.iter().take(5) {
                output::indent(&format!("• {} -> keys: {}", signature, keys.join(", ")), 2);
            }
        }
        output::info(&format!(
            "Run '{}' to repair malformed state entries",
            project_identity::cli_with("lint --repair-state")
        ));
    } else {
        output::success("State consistency: OK");
    }

    let now = chrono::Utc::now();
    let days_since_sync = (now - state.meta.last_sync).num_days();
    if days_since_sync > 7 {
        output::warning(&format!("Last sync was {} days ago", days_since_sync));
        output::info(&format!(
            "Consider running '{}' to update",
            project_identity::cli_with("sync")
        ));
    } else {
        output::success(&format!("Last sync: {} day(s) ago", days_since_sync));
    }

    // Summary
    output::separator();
    if all_ok {
        output::success("All checks passed!");
        output::info(&format!("Available backends: {}", available_backends.len()));
        if verbose {
            for backend in &available_backends {
                output::indent(&format!("• {}", backend), 2);
            }
        }
    } else {
        output::warning("Some issues found - see details above");
    }

    maybe_print_update_notification();

    Ok(())
}

fn maybe_print_update_notification() {
    let Some(hint) = update_hint_cached() else {
        return;
    };

    output::separator();
    output::warning(&format!(
        "New {} release available: {} -> {}",
        project_identity::BINARY_NAME,
        hint.current,
        hint.latest
    ));

    if is_managed_by_package_manager(&hint.owner) {
        let msg = match hint.owner {
            InstallOwner::Pacman => format!(
                "Update using package manager: paru -Syu {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Homebrew => format!(
                "Update using package manager: brew upgrade {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Scoop => format!(
                "Update using package manager: scoop update {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Winget => format!(
                "Update using package manager: winget upgrade {}",
                project_identity::BINARY_NAME
            ),
            _ => "Update using your package manager".to_string(),
        };
        output::info(&msg);
    } else {
        output::info(&format!(
            "For script/manual install, run: {} self-update",
            project_identity::BINARY_NAME
        ));
    }
}

/// Check backends dynamically from config
fn check_backends_dynamically(verbose: bool) -> Result<Vec<String>> {
    let mut available = Vec::new();
    let runtime_config = load_runtime_config_for_command("doctor backend checks");

    // Load backend configs (import-based or legacy)
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

fn collect_state_signature_duplicates(
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

#[cfg(test)]
mod tests;
