use crate::config::loader;
use crate::core::types::Backend;
use crate::error::Result;
use crate::state;
use crate::utils::paths;
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

#[allow(clippy::too_many_arguments)]
pub fn run(
    _verbose: bool,
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

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(crate::error::DeclarchError::ConfigNotFound { path: config_path });
    }

    // Handle --modules flag (load additional modules)
    let config = if !extra_modules.is_empty() {
        load_config_with_modules(&config_path, &extra_modules)?
    } else {
        loader::load_root_config(&config_path)?
    };

    let config_time = start_time.elapsed();

    println!("Configuration OK");

    // Handle --validate flag (exit early after validation)
    if validate_only {
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
        let backend = Backend::from_str(backend_str).map_err(crate::error::DeclarchError::ConfigError)?;
        let filtered_count = config
            .packages
            .iter()
            .filter(|(pkg_id, _)| pkg_id.backend == backend)
            .count();

        if filtered_count == 0 {
            println!("No packages found for backend: {}", backend_str);
            return Ok(());
        }

        filtered_count
    } else {
        config.packages.len()
    };

    println!("  Packages: {}", package_count);

    // Handle selective validation flags
    let check_dups = check_duplicates || only_duplicates;
    let check_confs = check_conflicts || only_conflicts;

    // Skip these checks if both only flags are set (mutually exclusive)
    if only_duplicates && only_conflicts {
        println!("Cannot check both duplicates and conflicts together");
        return Ok(());
    }

    if check_dups {
        let duplicates = config.get_duplicates();

        if duplicates.is_empty() {
            println!("No duplicates found");
        } else {
            println!("Found {} duplicates:", duplicates.len());
            for (pkg, sources) in duplicates {
                println!("  {}", pkg);
                for src in sources {
                    let display_path = src.file_name().unwrap_or_default().to_string_lossy();
                    println!("    in {}", display_path);
                }
            }
        }
    }

    if !only_duplicates && check_confs {

        let conflicts = config.get_cross_backend_conflicts();

        if conflicts.is_empty() {
            println!("No conflicts found");
        } else {
            println!("Found {} conflicts:", conflicts.len());
            for (pkg_name, backends) in conflicts {
                let backend_names: Vec<String> = backends.iter().map(|b| b.to_string()).collect();
                println!("  {} in backends: {}", pkg_name, backend_names.join(", "));
            }
        }
    }

    Ok(())
}

/// Show planned changes (diff between config and state)
fn show_diff(config: &loader::MergedConfig) -> Result<()> {
    use crate::core::types::PackageId;

    // Load state if available
    let state_path = state::io::get_state_path()?;
    if !state_path.exists() {
        println!("Packages to install:");
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
        println!("No changes planned");
        return Ok(());
    }

    if !to_install.is_empty() {
        println!("To install:");
        for pkg_id in &to_install {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    if !to_remove.is_empty() {
        println!("To remove:");
        for pkg_id in &to_remove {
            println!("  - {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    Ok(())
}

/// Show benchmark metrics
fn show_benchmarks(_config_time: std::time::Duration, total_time: std::time::Duration) {
    println!("Time: {:?}", total_time);
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

        // Load the module (silently)

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
