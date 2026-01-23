use crate::config::loader;
use crate::core::types::Backend;
use crate::error::Result;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;

/// Parse backend string to Backend enum
fn parse_backend(backend_str: &str) -> Result<Backend> {
    let backend_lower = backend_str.to_lowercase();
    match backend_lower.as_str() {
        "aur" => Ok(Backend::Aur),
        "flatpak" => Ok(Backend::Flatpak),
        "soar" => Ok(Backend::Soar),
        "npm" => Ok(Backend::Npm),
        "yarn" => Ok(Backend::Yarn),
        "pnpm" => Ok(Backend::Pnpm),
        "bun" => Ok(Backend::Bun),
        "pip" => Ok(Backend::Pip),
        "cargo" => Ok(Backend::Cargo),
        "brew" => Ok(Backend::Brew),
        _ => Ok(Backend::Custom(backend_str.to_string())),
    }
}

pub fn run(
    verbose: bool,
    check_duplicates: bool,
    check_conflicts: bool,
    backend_filter: Option<String>,
) -> Result<()> {
    output::header("Configuration Check");

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::error::DeclarchError::ConfigNotFound { path: config_path });
    }

    output::info(&format!("Entry point: {}", config_path.display()));

    let config = loader::load_root_config(&config_path)?;
    output::success("Syntax & Imports: OK");

    // Filter packages by backend if specified
    let package_count = if let Some(backend_str) = &backend_filter {
        let backend = parse_backend(backend_str)?;
        let filtered_count = config
            .packages
            .iter()
            .filter(|(pkg_id, _)| pkg_id.backend == backend)
            .count();

        if filtered_count == 0 {
            output::warning(&format!("No packages found for backend: {}", backend_str));
            return Ok(());
        }

        output::info(&format!("Filtering by backend: {}", backend_str));
        filtered_count
    } else {
        config.packages.len()
    };

    output::keyval("Unique Packages", &package_count.to_string());
    output::keyval("Excludes", &config.excludes.len().to_string());
    if verbose {
        output::separator();
        println!("{}", "Resolved Packages:".bold());

        let mut sorted_pkgs: Vec<_> = if let Some(backend_str) = &backend_filter {
            let backend = parse_backend(backend_str)?;
            config
                .packages
                .iter()
                .filter(|(pkg_id, _)| pkg_id.backend == backend)
                .map(|(pkg_id, _)| pkg_id)
                .collect()
        } else {
            config.packages.keys().collect()
        };

        sorted_pkgs.sort_by_key(|p| &p.name);

        for pkg in sorted_pkgs {
            output::indent(&pkg.to_string(), 2);
        }

        if !config.excludes.is_empty() {
            println!("\n{}", "Active Excludes:".bold());
            for ex in &config.excludes {
                output::indent(ex, 2);
            }
        }
    }
    if check_duplicates {
        output::separator();
        output::info("Checking for duplicates...");

        let duplicates = config.get_duplicates();

        if duplicates.is_empty() {
            output::success("No duplicate declarations found.");
        } else {
            output::warning(&format!(
                "Found {} duplicate package declarations:",
                duplicates.len()
            ));

            for (pkg, sources) in duplicates {
                println!("  📦 {}", pkg.to_string().yellow().bold());
                for src in sources {
                    let display_path = src.file_name().unwrap_or_default().to_string_lossy();
                    println!("     └─ {}", display_path.dimmed());
                }
            }
            println!(
                "\n{}",
                "Note: Duplicates are automatically deduplicated during sync.".italic()
            );
        }
    }

    if check_conflicts {
        output::separator();
        output::info("Checking for cross-backend conflicts...");

        let conflicts = config.get_cross_backend_conflicts();

        if conflicts.is_empty() {
            output::success("No cross-backend package name conflicts found.");
        } else {
            output::warning(&format!(
                "Found {} package name conflicts across backends:",
                conflicts.len()
            ));
            println!(
                "\n{}",
                "These packages have the same name but different backends:".bold()
            );
            println!(
                "{}",
                "They will be installed separately by each backend.".italic()
            );
            println!("{}", "Watch out for PATH conflicts!\n".dimmed());

            for (pkg_name, backends) in conflicts {
                println!("  ⚠️  {}", pkg_name.cyan().bold());
                for backend in &backends {
                    println!("     └─ {}", backend.to_string().yellow());
                }
            }

            println!("\n{}", "Example:".bold());
            println!("  If 'claude-cli' exists in both AUR and npm:");
            println!("    • AUR installs to: {}", "/usr/bin/claude-cli".dimmed());
            println!(
                "    • npm installs to:  {}",
                "~/.npm-global/bin/claude-cli".dimmed()
            );
            println!("  The one that runs depends on your {}", "PATH".bold());
            println!("\n  Use {}", "declarch info".bold().cyan());
            println!("  to see which backends have installed which packages.");
        }
    }

    Ok(())
}
