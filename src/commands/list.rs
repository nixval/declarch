mod filters;
mod output_formats;
mod unmanaged_output;

use crate::config::loader;
use crate::core::types::Backend;
use crate::error::Result;
use crate::project_identity;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use filters::{find_orphans, find_synced};
use output_formats::{output_json, output_yaml};
use std::collections::HashMap;
use std::str::FromStr;
use unmanaged_output::emit_unmanaged_output;

/// Options for the list command
pub struct ListOptions {
    pub backend: Option<String>,
    pub orphans: bool,
    pub synced: bool,
    pub unmanaged: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
}

#[derive(serde::Serialize)]
pub(super) struct UnmanagedPackageOut {
    backend: String,
    name: String,
    version: Option<String>,
}

pub fn run(options: ListOptions) -> Result<()> {
    if options.unmanaged {
        return run_unmanaged_list(options);
    }

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
        let backend =
            Backend::from_str(backend_str).map_err(crate::error::DeclarchError::ConfigError)?;
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
        "json" => output_json(&packages, options.output_version.as_deref()),
        "yaml" => output_yaml(&packages, options.output_version.as_deref()),
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

fn run_unmanaged_list(options: ListOptions) -> Result<()> {
    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::error::DeclarchError::ConfigNotFound { path: config_path });
    }
    let config = loader::load_root_config(&config_path)?;
    let declared = config.packages;

    let mut backend_configs = crate::backends::load_all_backends_unified()?;
    if let Some(backend_filter) = &options.backend {
        backend_configs.retain(|name, _| name == backend_filter);
    }

    let mut out: Vec<UnmanagedPackageOut> = Vec::new();
    for (name, cfg) in backend_configs {
        if !crate::utils::platform::backend_supports_current_os(&cfg) {
            continue;
        }
        if cfg.list_cmd.is_none() {
            continue;
        }
        let manager: Box<dyn crate::packages::traits::PackageManager> = Box::new(
            crate::backends::GenericManager::from_config(cfg, Backend::from(name.as_str()), false),
        );
        if !manager.is_available() {
            continue;
        }

        let installed = match manager.list_installed() {
            Ok(pkgs) => pkgs,
            Err(e) => {
                output::warning(&format!("{}: {}", name, e));
                continue;
            }
        };

        for (pkg_name, meta) in installed {
            let pkg_id = crate::core::types::PackageId {
                backend: Backend::from(name.as_str()),
                name: pkg_name.clone(),
            };
            if !declared.contains_key(&pkg_id) {
                out.push(UnmanagedPackageOut {
                    backend: name.clone(),
                    name: pkg_name,
                    version: meta.version,
                });
            }
        }
    }

    out.sort_by(|a, b| a.backend.cmp(&b.backend).then(a.name.cmp(&b.name)));
    out.dedup_by(|a, b| a.backend == b.backend && a.name == b.name);

    emit_unmanaged_output(&out, &options)
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

    // Sort backends alphabetically for consistent display
    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort_by(|a, b| a.name().cmp(b.name()));

    for backend in &backends {
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

    // Note: All backends are now displayed in the sorted loop above
    // No hardcoded backend order - everything is dynamic

    if is_orphans {
        println!();
        output::info(&format!(
            "Orphan packages are not managed by {}",
            project_identity::BINARY_NAME
        ));
        output::info(&format!(
            "Add them to your config or use '{}' to remove",
            project_identity::cli_with("sync prune")
        ));
    }
}
