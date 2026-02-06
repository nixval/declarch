use crate::config::loader;
use crate::core::types::Backend;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

#[allow(clippy::too_many_arguments)]
pub fn run(
    verbose: bool,
    check_duplicates: bool,
    check_conflicts: bool,
    only_duplicates: bool,
    only_conflicts: bool,
    backend_filter: Option<String>,
    diff: bool,
    validate_only: bool,
    benchmark: bool,
    extra_modules: Vec<String>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    output::header("Configuration Check");

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::error::DeclarchError::ConfigNotFound { path: config_path });
    }

    // Handle --modules flag (load additional modules)
    let config = if !extra_modules.is_empty() {
        output::info(&format!("Loading additional modules: {:?}", extra_modules));
        load_config_with_modules(&config_path, &extra_modules)?
    } else {
        loader::load_root_config(&config_path)?
    };

    let config_time = start_time.elapsed();

    output::success("Syntax & Imports: OK");

    // Handle --validate flag (exit early after validation)
    if validate_only {
        output::info("Validation complete (no sync performed)");
        if benchmark {
            show_benchmarks(config_time, start_time.elapsed());
        }
        return Ok(());
    }

    // Handle --diff flag (show planned changes)
    if diff {
        show_diff(&config)?;
    }

    // Handle --benchmark flag
    if benchmark {
        show_benchmarks(config_time, start_time.elapsed());
    }

    // Filter packages by backend if specified
    let package_count = if let Some(backend_str) = &backend_filter {
        let backend = Backend::from_str(backend_str).map_err(|e| crate::error::DeclarchError::ConfigError(e))?;
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
            let backend = Backend::from_str(backend_str).map_err(|e| crate::error::DeclarchError::ConfigError(e))?;
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

    // Handle selective validation flags
    let check_dups = check_duplicates || only_duplicates;
    let check_confs = check_conflicts || only_conflicts;

    // Skip these checks if both only flags are set (mutually exclusive)
    if only_duplicates && only_conflicts {
        output::warning("Cannot specify both --only-duplicates and --only-conflicts");
        output::info("Use --duplicates --conflicts to check both");
        return Ok(());
    }

    if check_dups {
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

    // Skip conflicts check if only_duplicates is set
    if !only_duplicates && check_confs {
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

/// Show planned changes (diff between config and state)
fn show_diff(config: &loader::MergedConfig) -> Result<()> {
    use crate::core::types::PackageId;

    output::separator();
    output::info("Calculating planned changes...");

    // Load state if available
    let state_path = state::io::get_state_path()?;
    if !state_path.exists() {
        output::warning("No state found. All packages will be installed.");
        output::separator();
        println!("{}", "Packages to Install:".bold().green());

        for pkg_id in config.packages.keys() {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
        return Ok(());
    }

    let state = state::io::load_state()?;

    // Create sets for comparison
    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
    let mut state_set: HashSet<PackageId> = HashSet::new();

    for pkg_state in state.packages.values() {
        let pkg_id = PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        };
        state_set.insert(pkg_id);
    }

    // Calculate differences
    let to_install: Vec<_> = config_set.difference(&state_set).cloned().collect();

    let to_remove: Vec<_> = state_set.difference(&config_set).cloned().collect();

    // Display results
    if to_install.is_empty() && to_remove.is_empty() {
        output::success("No changes planned. System is in sync.");
        return Ok(());
    }

    if !to_install.is_empty() {
        println!();
        println!(
            "{}",
            format!("To Install ({}):", to_install.len()).bold().green()
        );
        for pkg_id in &to_install {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    if !to_remove.is_empty() {
        println!();
        println!(
            "{}",
            format!("To Remove ({}):", to_remove.len()).bold().red()
        );
        for pkg_id in &to_remove {
            println!("  - {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    println!();
    output::info("Run 'dcl sync' to apply these changes");

    Ok(())
}

/// Show benchmark metrics
fn show_benchmarks(config_time: std::time::Duration, total_time: std::time::Duration) {
    println!();
    println!("{}", "Performance Metrics:".bold());

    println!("  Config loading:     {:>8} ms", config_time.as_millis());
    println!("  Total time:         {:>8} ms", total_time.as_millis());
}

/// Load config with additional modules
fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    // Load base config
    let mut merged = loader::load_root_config(config_path)?;

    // Load each additional module
    for module_name in extra_modules {
        // Try as module name (e.g., "gaming" -> modules/gaming.kdl)
        let module_path = paths::module_file(module_name);

        let final_path = if let Ok(path) = module_path {
            if path.exists() {
                path
            } else {
                // Try as direct path
                let direct_path = PathBuf::from(module_name);
                if direct_path.exists() {
                    direct_path
                } else {
                    return Err(crate::error::DeclarchError::Other(format!(
                        "Module not found: {}",
                        module_name
                    )));
                }
            }
        } else {
            // Try as direct path
            let direct_path = PathBuf::from(module_name);
            if direct_path.exists() {
                direct_path
            } else {
                return Err(crate::error::DeclarchError::Other(format!(
                    "Module not found: {}",
                    module_name
                )));
            }
        };

        // Load the module
        output::info(&format!("  Loading module: {}", final_path.display()));

        // Use internal loader function (it's actually public via load_root_config, but we need to work with the merged config)
        // We'll use load_root_config on the module file directly
        let module_config = loader::load_root_config(&final_path)?;

        // Merge the module config into our existing config
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
        // Note: We're not merging other fields like meta, conflicts, etc. to keep it simple
    }

    Ok(merged)
}
