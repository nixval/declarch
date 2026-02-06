use crate::config::loader;
use crate::core::types::Backend;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use std::collections::HashMap;
use std::str::FromStr;

/// Options for the list command
pub struct ListOptions {
    pub backend: Option<String>,
    pub orphans: bool,
    pub synced: bool,
    pub format: Option<String>,
}

pub fn run(options: ListOptions) -> Result<()> {
    let state = state::io::load_state()?;

    // Load config for orphan/synced detection
    let config_packages = if options.orphans || options.synced {
        let config_path = paths::config_file()?;
        if config_path.exists() {
            let config = loader::load_root_config(&config_path)?;
            Some(config.packages)
        } else {
            None
        }
    } else {
        None
    };

    // Collect and filter packages
    let mut packages: Vec<_> = state.packages.values().collect();

    // Filter by backend if specified
    if let Some(backend_str) = &options.backend {
        let backend = Backend::from_str(backend_str).map_err(crate::error::DeclarchError::ConfigError)?;
        packages.retain(|p| p.backend == backend);
    }

    // Filter orphans
    if options.orphans {
        if let Some(config_pkgs) = &config_packages {
            packages = find_orphans(packages, config_pkgs);
        } else {
            output::warning("No config found, cannot determine orphans");
        }
    }

    // Filter synced
    if options.synced {
        if let Some(config_pkgs) = &config_packages {
            packages = find_synced(packages, config_pkgs);
        } else {
            output::warning("No config found, cannot determine synced packages");
        }
    }

    // Determine output format
    let format_str = options.format.as_deref().unwrap_or("table");

    match format_str {
        "json" => output_json(&packages),
        "yaml" => output_yaml(&packages),
        "table" => {
            let total = packages.len();
            display_packages(&packages, options.orphans, total);
            Ok(())
        }
        _ => {
            let total = packages.len();
            display_packages(&packages, options.orphans, total);
            Ok(())
        }
    }
}



/// Find orphan packages (installed but not in config)
fn find_orphans<'a>(
    installed: Vec<&'a state::types::PackageState>,
    config: &HashMap<crate::core::types::PackageId, Vec<std::path::PathBuf>>,
) -> Vec<&'a state::types::PackageState> {
    installed
        .into_iter()
        .filter(|pkg| {
            // Create PackageId for lookup
            let pkg_id = crate::core::types::PackageId {
                backend: pkg.backend.clone(),
                name: pkg.config_name.clone(),
            };
            !config.contains_key(&pkg_id)
        })
        .collect()
}

/// Find synced packages (installed and in config)
fn find_synced<'a>(
    installed: Vec<&'a state::types::PackageState>,
    config: &HashMap<crate::core::types::PackageId, Vec<std::path::PathBuf>>,
) -> Vec<&'a state::types::PackageState> {
    installed
        .into_iter()
        .filter(|pkg| {
            let pkg_id = crate::core::types::PackageId {
                backend: pkg.backend.clone(),
                name: pkg.config_name.clone(),
            };
            config.contains_key(&pkg_id)
        })
        .collect()
}

/// Display packages with formatting
fn display_packages(packages: &[&state::types::PackageState], is_orphans: bool, total: usize) {
    if packages.is_empty() {
        output::info("No packages found");
        return;
    }

    // Group by backend
    let mut grouped: HashMap<Backend, Vec<&state::types::PackageState>> = HashMap::new();
    for pkg in packages.iter() {
        grouped.entry(pkg.backend.clone()).or_default().push(*pkg);
    }

    output::header(&format!("Installed Packages ({})", total));

    // Display by backend
    let backend_order = vec![
        Backend::from("aur"),
        Backend::from("flatpak"),
        Backend::from("cargo"),
        Backend::from("npm"),
        Backend::from("bun"),
        Backend::from("yarn"),
        Backend::from("pnpm"),
        Backend::from("pip"),
        Backend::from("brew"),
        Backend::from("soar"),
    ];

    // Handle other backends not in the predefined order
    let known_backends: std::collections::HashSet<_> = backend_order.iter().collect();

    for backend in &backend_order {
        if let Some(pkgs) = grouped.get(backend) {
            println!();
            println!("{}", format!("Backend: {}", backend).bold().cyan());

            for pkg in pkgs {
                let status = if is_orphans {
                    "⚠".yellow()
                } else {
                    "✓".green()
                };

                let version = pkg.version.as_ref().map(|v| v as &str).unwrap_or("-");
                println!(
                    "  {} {:<30} {:>10}",
                    status,
                    &pkg.config_name,
                    version.dimmed()
                );
            }
        }
    }

    // Handle other backends not in the predefined order
    for (backend, pkgs) in grouped.iter() {
        if !known_backends.contains(backend) {
            println!();
            println!("{}", format!("Backend: {}", backend).bold().cyan());
            for pkg in pkgs {
                let status = if is_orphans {
                    "⚠".yellow()
                } else {
                    "✓".green()
                };
                let version = pkg.version.as_ref().map(|v| v as &str).unwrap_or("-");
                println!(
                    "  {} {:<30} {:>10}",
                    status,
                    &pkg.config_name,
                    version.dimmed()
                );
            }
        }
    }

    if is_orphans {
        println!();
        output::info("Orphan packages are not managed by declarch");
        output::info("Add them to your config or use 'dcl sync --prune' to remove");
    }
}

/// Output packages as JSON
fn output_json(packages: &[&state::types::PackageState]) -> Result<()> {
    let json = serde_json::to_string_pretty(packages)?;
    println!("{}", json);
    Ok(())
}

/// Output packages as YAML
fn output_yaml(packages: &[&state::types::PackageState]) -> Result<()> {
    let json_value = serde_json::to_value(packages)?;
    let yaml = serde_yaml::to_string(&json_value)?;
    println!("{}", yaml);
    Ok(())
}
