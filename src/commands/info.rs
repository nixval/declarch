use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::config::loader;
use crate::error::Result;
use crate::packages::traits::PackageManager;
use crate::state;
use crate::ui as output;
use crate::utils::machine_output;
use crate::utils::paths;
use crate::utils::update_check::{InstallOwner, is_managed_by_package_manager, update_hint_cached};
use colored::Colorize;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};

/// Indentation width used for package display formatting
const PACKAGE_INDENT_WIDTH: usize = 4;
/// Prefix length reserved for backend name in package listing
const BACKEND_PREFIX_LEN: usize = 10;
/// Indentation for line continuation in multi-line package lists
const CONTINUATION_INDENT: &str = "     ";

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
    output::header("System Status");
    output::keyval("Hostname", &state.meta.hostname.cyan().bold().to_string());
    output::keyval(
        "Last Sync",
        &state.meta.last_sync.format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    let pkg_count = filtered_packages.len();
    let backend_counts = count_backends_filtered(filtered_packages);

    println!();
    output::tag("Total Managed", &pkg_count.to_string());

    // Dynamic backend display - no hardcoded list
    let mut backends: Vec<_> = backend_counts.iter().collect();
    backends.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

    for (backend_key, count) in backends {
        if *count > 0 {
            output::indent(&format!("• {}: {}", backend_key, count), 2);
        }
    }

    if pkg_count > 0 {
        output::separator();
        println!("{}", "Managed Packages:".bold());

        let mut sorted_packages: Vec<_> = filtered_packages.to_vec();
        sorted_packages
            .sort_by(|(k1, _), (k2, _)| extract_package_name(k1).cmp(extract_package_name(k2)));

        print_packages_horizontally(sorted_packages);
    }

    Ok(())
}

fn output_json_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    let packages: Vec<&state::types::PackageState> =
        filtered_packages.iter().map(|(_, pkg)| *pkg).collect();

    if output_version == Some("v1") {
        return machine_output::emit_v1("info", &packages, Vec::new(), Vec::new(), "json");
    }

    let json = serde_json::to_string_pretty(&packages)?;
    println!("{}", json);
    Ok(())
}

fn output_yaml_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    let packages: Vec<&state::types::PackageState> =
        filtered_packages.iter().map(|(_, pkg)| *pkg).collect();

    if output_version == Some("v1") {
        return machine_output::emit_v1("info", &packages, Vec::new(), Vec::new(), "yaml");
    }

    let json_value = serde_json::to_value(&packages)?;
    let yaml = serde_yml::to_string(&json_value)?;
    println!("{}", yaml);
    Ok(())
}

fn extract_package_name(key: &str) -> &str {
    key.split_once(':').map(|(_, n)| n).unwrap_or(key)
}

fn count_backends_filtered(
    packages: &[(&String, &state::types::PackageState)],
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for (_, package) in packages {
        let backend_name = package.backend.name();
        *counts.entry(backend_name.to_string()).or_insert(0) += 1;
    }
    counts
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
        output::info("Run 'declarch init' to create a configuration");
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
                        output::info("Run 'declarch info --list --scope orphans' to see them");
                        output::info("Run 'declarch sync prune' to remove orphans");
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
        output::info("Run 'declarch sync' to create initial state");
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

    let keys: Vec<_> = state.packages.keys().collect();
    let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
    if keys.len() != unique_keys.len() {
        output::warning("State has duplicate keys - consider running sync to fix");
    } else {
        output::success("State consistency: OK");
    }

    let now = chrono::Utc::now();
    let days_since_sync = (now - state.meta.last_sync).num_days();
    if days_since_sync > 7 {
        output::warning(&format!("Last sync was {} days ago", days_since_sync));
        output::info("Consider running 'declarch sync' to update");
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
        "New declarch release available: {} -> {}",
        hint.current, hint.latest
    ));

    if is_managed_by_package_manager(&hint.owner) {
        let msg = match hint.owner {
            InstallOwner::Pacman => "Update using package manager: paru -Syu declarch",
            InstallOwner::Homebrew => "Update using package manager: brew upgrade declarch",
            InstallOwner::Scoop => "Update using package manager: scoop update declarch",
            InstallOwner::Winget => "Update using package manager: winget upgrade declarch",
            _ => "Update using your package manager",
        };
        output::info(msg);
    } else {
        output::info("For curl/manual install, run: declarch self-update");
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
        output::info("Run 'declarch init --backend <name>' to add a backend");
    }

    Ok(available)
}

/// Print packages grouped by backend with horizontal display per group
fn print_packages_horizontally(packages: Vec<(&String, &state::types::PackageState)>) {
    if packages.is_empty() {
        return;
    }

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    // Group by backend
    let mut grouped: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (_, pkg_state) in &packages {
        let backend_name = pkg_state.backend.to_string();
        grouped
            .entry(backend_name)
            .or_default()
            .push(pkg_state.config_name.clone());
    }

    // Sort backends alphabetically for consistent display
    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort();

    // Display each backend group
    for backend in backends {
        if let Some(pkg_names) = grouped.get(&backend)
            && !pkg_names.is_empty()
        {
            println!(
                "  {}: {}",
                backend.bold().cyan(),
                format_packages_inline(pkg_names, term_width)
            );
        }
    }
}

fn format_packages_inline(pkg_names: &[String], term_width: usize) -> String {
    let available_width = term_width.saturating_sub(BACKEND_PREFIX_LEN + PACKAGE_INDENT_WIDTH);

    if available_width < 20 {
        return pkg_names.join(" ");
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut line_length = 0;

    for pkg_name in pkg_names {
        let item_length = pkg_name.len() + 2;

        if line_length + item_length > available_width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line.clear();
            line_length = 0;
        }

        if !current_line.is_empty() {
            current_line.push_str("  ");
            line_length += 2;
        }
        current_line.push_str(pkg_name);
        line_length += pkg_name.len();
    }

    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    if lines.len() > 1 {
        lines.join(&format!("\n{CONTINUATION_INDENT}"))
    } else {
        lines.join("\n")
    }
}
