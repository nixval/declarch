mod summary;

use crate::config::loader;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;
use terminal_size::{Width, terminal_size};

pub struct InfoOptions {
    pub doctor: bool,
    pub debug: bool,
    pub format: Option<String>,
    pub backend: Option<String>,
    pub package: Option<String>,
    pub summary: bool,
}

pub fn run(options: InfoOptions) -> Result<()> {
    // Handle --debug flag (must be set early)
    // Note: std::env::set_var is unsafe in Rust 1.92+ for safety reasons.
    // This is safe here because we're setting the variable before any threads are spawned
    // and we're only setting it at process startup.
    if options.debug {
        unsafe { std::env::set_var("RUST_LOG", "debug") };
        output::info("Debug logging enabled");
    }

    // Handle --doctor flag
    if options.doctor {
        return run_doctor();
    }

    // Default info behavior
    run_info(&options)
}

fn run_info(options: &InfoOptions) -> Result<()> {
    // Handle --summary flag for quick overview
    if options.summary {
        return summary::display_summary();
    }

    let state = state::io::load_state()?;

    // Apply filters
    let filtered_packages: Vec<(&String, &state::types::PackageState)> =
        if options.backend.is_some() || options.package.is_some() {
            let backend_filter = options.backend.as_deref();
            let package_filter = options.package.as_deref();

            state
                .packages
                .iter()
                .filter(|(key, pkg_state)| {
                    let name = extract_package_name(key);

                    // Filter by package name if specified
                    let Some(filter_pkg) = package_filter else {
                        return true;
                    };
                    if !name.contains(filter_pkg) {
                        return false;
                    }

                    // Filter by backend if specified
                    if let Some(filter_backend) = backend_filter {
                        // Check if package backend matches filter
                        match (&pkg_state.backend, filter_backend) {
                            (crate::state::types::Backend::Aur, "aur") => true,
                            (crate::state::types::Backend::Flatpak, "flatpak") => true,
                            (crate::state::types::Backend::Soar, "soar") => true,
                            (crate::state::types::Backend::Npm, "npm") => true,
                            (crate::state::types::Backend::Yarn, "yarn") => true,
                            (crate::state::types::Backend::Pnpm, "pnpm") => true,
                            (crate::state::types::Backend::Bun, "bun") => true,
                            (crate::state::types::Backend::Pip, "pip") => true,
                            (crate::state::types::Backend::Cargo, "cargo") => true,
                            (crate::state::types::Backend::Brew, "brew") => true,
                            (crate::state::types::Backend::Custom(pkg_name), filter_name) => {
                                pkg_name == filter_name
                            }
                            _ => false,
                        }
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            state.packages.iter().collect()
        };

    // Determine output format
    let format_str = options.format.as_deref().unwrap_or("table");

    match format_str {
        "json" => output_json_filtered(&filtered_packages, &state),
        "yaml" => output_yaml_filtered(&filtered_packages, &state),
        "table" => output_table_filtered(&state, &filtered_packages),
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

    // Optimized: Count all backends in single pass
    let backend_counts = count_backends_filtered(filtered_packages);

    println!();
    output::tag("Total Managed", &pkg_count.to_string());

    // Only show backends that have packages
    let backends_to_show = vec![
        ("aur", "AUR/Repo"),
        ("flatpak", "Flatpak"),
        ("soar", "Soar"),
        ("npm", "NPM"),
        ("yarn", "Yarn"),
        ("pnpm", "Pnpm"),
        ("bun", "Bun"),
        ("pip", "Pip"),
        ("cargo", "Cargo"),
        ("brew", "Brew"),
    ];

    for (backend_key, backend_name) in backends_to_show {
        let Some(&count) = backend_counts.get(backend_key) else {
            continue;
        };
        if count > 0 {
            output::indent(&format!("• {}: {}", backend_name, count), 2);
        }
    }

    if pkg_count > 0 {
        output::separator();
        println!("{}", "Managed Packages:".bold());

        // Sort by name using helper function
        let mut sorted_packages: Vec<_> = filtered_packages.to_vec();
        sorted_packages
            .sort_by(|(k1, _), (k2, _)| extract_package_name(k1).cmp(extract_package_name(k2)));

        // Display packages horizontally with backend prefix
        print_packages_horizontally(sorted_packages);
    }

    Ok(())
}

fn output_json_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    _state: &state::types::State,
) -> Result<()> {
    let packages: Vec<&state::types::PackageState> =
        filtered_packages.iter().map(|(_, pkg)| *pkg).collect();

    let json = serde_json::to_string_pretty(&packages)?;
    println!("{}", json);
    Ok(())
}

fn output_yaml_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    _state: &state::types::State,
) -> Result<()> {
    let packages: Vec<&state::types::PackageState> =
        filtered_packages.iter().map(|(_, pkg)| *pkg).collect();

    let json_value = serde_json::to_value(&packages)?;
    let yaml = serde_yaml::to_string(&json_value)?;
    println!("{}", yaml);
    Ok(())
}

/// Helper function to extract package name from key "backend:name"
fn extract_package_name(key: &str) -> &str {
    key.split_once(':').map(|(_, n)| n).unwrap_or(key)
}

/// Count backends from filtered package references
fn count_backends_filtered(
    packages: &[(&String, &state::types::PackageState)],
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for (_, package) in packages {
        let backend_name = match &package.backend {
            crate::state::types::Backend::Aur => "aur",
            crate::state::types::Backend::Flatpak => "flatpak",
            crate::state::types::Backend::Soar => "soar",
            crate::state::types::Backend::Npm => "npm",
            crate::state::types::Backend::Yarn => "yarn",
            crate::state::types::Backend::Pnpm => "pnpm",
            crate::state::types::Backend::Bun => "bun",
            crate::state::types::Backend::Pip => "pip",
            crate::state::types::Backend::Cargo => "cargo",
            crate::state::types::Backend::Brew => "brew",
            crate::state::types::Backend::Custom(name) => name.as_str(),
        };
        *counts.entry(backend_name.to_string()).or_insert(0) += 1;
    }
    counts
}

fn run_doctor() -> Result<()> {
    output::header("System Diagnosis");

    let mut all_ok = true;

    // Check 1: Config file
    output::info("Checking configuration file...");
    let config_path = paths::config_file()?;
    if config_path.exists() {
        output::success(&format!("Config found: {}", config_path.display()));

        // Try to parse it
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
        output::warning(&format!("Config not found: {}", config_path.display()));
        output::info("Run 'dcl init' to create a configuration");
        all_ok = false;
    }

    // Check 2: State file
    output::info("Checking state file...");
    let state_path = state::io::get_state_path()?;
    if state_path.exists() {
        output::success(&format!("State found: {}", state_path.display()));

        // Try to load it
        match state::io::load_state() {
            Ok(state) => {
                output::success(&format!(
                    "State valid: {} packages tracked",
                    state.packages.len()
                ));

                // Check for orphans
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
                        output::info("Run 'dcl list --orphans' to see them");
                        output::info("Run 'dcl sync --prune' to remove orphans");
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
        output::info("Run 'dcl sync' to create initial state");
    }

    // Check 3: Backend availability
    output::info("Checking backends...");
    let backends_to_check = vec![
        ("paru", "AUR (paru)"),
        ("yay", "AUR (yay)"),
        ("flatpak", "Flatpak"),
        ("cargo", "Cargo"),
        ("npm", "npm"),
        ("bun", "Bun"),
        ("pip", "pip"),
    ];

    let mut available_backends = 0;
    for (cmd, name) in &backends_to_check {
        if is_command_available(cmd) {
            output::success(&format!("{}: Available", name));
            available_backends += 1;
        } else {
            output::warning(&format!("{}: Not found", name));
        }
    }

    // Check 4: State consistency
    output::info("Checking state consistency...");
    let Some(state) = state_path
        .exists()
        .then(state::io::load_state)
        .transpose()?
    else {
        return Ok(());
    };

    // Check for duplicate keys
    let keys: Vec<_> = state.packages.keys().collect();
    let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
    if keys.len() != unique_keys.len() {
        output::warning("State has duplicate keys - consider running sync to fix");
    } else {
        output::success("State consistency: OK");
    }

    // Check last sync time
    let now = chrono::Utc::now();
    let days_since_sync = (now - state.meta.last_sync).num_days();
    if days_since_sync > 7 {
        output::warning(&format!("Last sync was {} days ago", days_since_sync));
        output::info("Consider running 'dcl sync' to update");
    } else {
        output::success(&format!("Last sync: {} day(s) ago", days_since_sync));
    }

    // Summary
    output::separator();
    if all_ok {
        output::success("All checks passed!");
        output::info(&format!(
            "Available backends: {}/{}",
            available_backends,
            backends_to_check.len()
        ));
    } else {
        output::warning("Some issues found - see details above");
    }

    Ok(())
}

/// Print packages grouped by backend with horizontal display per group
fn print_packages_horizontally(packages: Vec<(&String, &state::types::PackageState)>) {
    if packages.is_empty() {
        return;
    }

    // Get terminal width, default to 80 if detection fails
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

    // Define order for known backends
    let backend_order = vec![
        "aur", "soar", "flatpak", "npm", "yarn", "pnpm", "bun", "pip", "cargo", "brew",
    ];

    // Display each backend group
    for backend in &backend_order {
        let Some(pkg_names) = grouped.get(*backend) else {
            continue;
        };
        if !pkg_names.is_empty() {
            println!(
                "  {}: {}",
                backend.bold().cyan(),
                format_packages_inline(pkg_names, term_width)
            );
        }
    }

    // Display any custom backends (not in predefined order)
    for (backend, pkg_names) in grouped {
        if !backend_order.contains(&backend.as_str()) && !pkg_names.is_empty() {
            println!(
                "  {}: {}",
                backend.bold().cyan(),
                format_packages_inline(&pkg_names, term_width)
            );
        }
    }
}

/// Format package names inline with auto-wrapping
fn format_packages_inline(pkg_names: &[String], term_width: usize) -> String {
    // Calculate available space (subtract "backend: " prefix and indentation)
    let prefix_len = 10; // "backend: " is roughly 10 chars with colors
    let available_width = term_width.saturating_sub(prefix_len + 4); // +4 for margin

    if available_width < 20 {
        // Too narrow, just return comma-separated
        return pkg_names.join(" ");
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut line_length = 0;

    for pkg_name in pkg_names {
        let item_length = pkg_name.len() + 2; // +2 for spacing between items

        // Check if we need to wrap to next line
        if line_length + item_length > available_width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line.clear();
            line_length = 0;
        }

        // Add spacing between items
        if !current_line.is_empty() {
            current_line.push_str("  ");
        }

        current_line.push_str(pkg_name);
        line_length += item_length;
    }

    // Add remaining items
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Join lines with proper indentation
    if lines.len() > 1 {
        lines.join("\n      ") // Indent continuation lines
    } else {
        lines.join("  ")
    }
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
