use crate::error::Result;
use crate::ui as output;
use crate::utils::machine_output;
use colored::Colorize;
use std::collections::HashMap;

use super::{ListOptions, UnmanagedPackageOut};

pub(super) fn emit_unmanaged_output(
    out: &[UnmanagedPackageOut],
    options: &ListOptions,
) -> Result<()> {
    let format_str = options.format.as_deref().unwrap_or("table");
    match format_str {
        "json" => emit_json(out, options.output_version.as_deref()),
        "yaml" => emit_yaml(out, options.output_version.as_deref()),
        _ => emit_table(out),
    }
}

fn emit_json(out: &[UnmanagedPackageOut], output_version: Option<&str>) -> Result<()> {
    if output_version == Some("v1") {
        machine_output::emit_v1("info --list", out, Vec::new(), Vec::new(), "json")?;
    } else {
        println!("{}", serde_json::to_string_pretty(out)?);
    }
    Ok(())
}

fn emit_yaml(out: &[UnmanagedPackageOut], output_version: Option<&str>) -> Result<()> {
    if output_version == Some("v1") {
        machine_output::emit_v1("info --list", out, Vec::new(), Vec::new(), "yaml")?;
    } else {
        let yaml = serde_yml::to_string(&serde_json::to_value(out)?)?;
        println!("{}", yaml);
    }
    Ok(())
}

fn emit_table(out: &[UnmanagedPackageOut]) -> Result<()> {
    if out.is_empty() {
        output::info("No unmanaged installed packages found");
        return Ok(());
    }

    output::header(&format!("Unmanaged Installed Packages ({})", out.len()));
    let mut by_backend: HashMap<String, Vec<&UnmanagedPackageOut>> = HashMap::new();
    for item in out {
        by_backend
            .entry(item.backend.clone())
            .or_default()
            .push(item);
    }
    let mut keys: Vec<_> = by_backend.keys().cloned().collect();
    keys.sort();
    for backend in keys {
        println!();
        println!("{}", format!("Backend: {}", backend).bold().cyan());
        if let Some(pkgs) = by_backend.get(&backend) {
            for pkg in pkgs {
                println!(
                    "  {} {:<30} {:>10}",
                    "•".yellow(),
                    pkg.name,
                    pkg.version.as_deref().unwrap_or("-").dimmed()
                );
            }
        }
    }
    println!();
    output::info("Tip: add needed packages into your config to adopt them.");
    Ok(())
}
