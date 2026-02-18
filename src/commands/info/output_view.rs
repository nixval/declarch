use crate::error::Result;
use crate::state;
use crate::ui as output;
use crate::utils::machine_output;
use colored::Colorize;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};

/// Indentation width used for package display formatting
const PACKAGE_INDENT_WIDTH: usize = 4;
/// Prefix length reserved for backend name in package listing
const BACKEND_PREFIX_LEN: usize = 10;
/// Indentation for line continuation in multi-line package lists
const CONTINUATION_INDENT: &str = "     ";

pub(super) fn output_table_filtered(
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
    backends.sort_by(|a, b| b.1.cmp(a.1));

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

pub(super) fn output_json_filtered(
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

pub(super) fn output_yaml_filtered(
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

pub(super) fn count_backends_filtered(
    packages: &[(&String, &state::types::PackageState)],
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for (_, package) in packages {
        let backend_name = package.backend.name();
        *counts.entry(backend_name.to_string()).or_insert(0) += 1;
    }
    counts
}

fn extract_package_name(key: &str) -> &str {
    key.split_once(':').map(|(_, n)| n).unwrap_or(key)
}

/// Print packages grouped by backend with horizontal display per group
fn print_packages_horizontally(packages: Vec<(&String, &state::types::PackageState)>) {
    if packages.is_empty() {
        return;
    }

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for (_, pkg_state) in &packages {
        let backend_name = pkg_state.backend.to_string();
        grouped
            .entry(backend_name)
            .or_default()
            .push(pkg_state.config_name.clone());
    }

    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort();

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
