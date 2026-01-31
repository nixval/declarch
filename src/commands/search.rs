//! Package search command
//!
//! Search for packages across multiple backends

use crate::core::types::{Backend, PackageId};
use crate::error::Result;
use crate::packages::traits::{PackageSearchResult, PackageManager};
use crate::state;
use crate::ui as output;
use colored::Colorize;

pub struct SearchOptions {
    pub query: String,
    pub backends: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub installed_only: bool,
    pub available_only: bool,
}

/// Parse query for optional "backend:query" syntax
/// Returns (backend_from_query, actual_query)
fn parse_backend_query(query: &str) -> (Option<String>, String) {
    if query.contains(':') {
        let parts: Vec<&str> = query.splitn(2, ':').collect();
        if parts.len() == 2 {
            let potential_backend = parts[0].trim();
            let actual_query = parts[1].trim();

            // Check if it looks like a backend name (no spaces, alphanumeric)
            if !potential_backend.contains(' ') && !potential_backend.is_empty() {
                return (Some(potential_backend.to_string()), actual_query.to_string());
            }
        }
    }
    (None, query.to_string())
}

pub fn run(options: SearchOptions) -> Result<()> {
    // Load state to check installed packages
    let state = state::io::load_state()?;

    // Parse "backend:query" syntax
    let (backend_from_query, actual_query) = parse_backend_query(&options.query);

    // Merge backend_from_query with options.backends
    let final_backends = if let Some(backend) = backend_from_query {
        // backend:query syntax overrides --backends flag
        Some(vec![backend])
    } else {
        options.backends.clone()
    };

    // Create updated options for internal use
    let updated_options = SearchOptions {
        query: actual_query.clone(),
        backends: final_backends,
        limit: options.limit,
        installed_only: options.installed_only,
        available_only: options.available_only,
    };

    // Get backends to search
    let backends_to_search = get_backends_to_search(&updated_options)?;

    if backends_to_search.is_empty() {
        output::warning("No backends available for search");
        return Ok(());
    }

    // Search across backends and collect results
    let mut all_results: Vec<PackageSearchResult> = Vec::new();

    for backend in backends_to_search {
        match create_manager_for_backend(&backend) {
            Ok(manager) if manager.supports_search() => {
                output::info(&format!("Searching {}...", backend));

                match manager.search(&actual_query) {
                    Ok(mut results) => {
                        // Apply result limiting
                        if let Some(limit_value) = updated_options.limit {
                            if results.len() > limit_value {
                                results.truncate(limit_value);
                            }
                        }

                        // Mark installed packages
                        for result in &mut results {
                            let pkg_id = PackageId {
                                name: result.name.clone(),
                                backend: result.backend.clone(),
                            };
                            let state_key = crate::core::resolver::make_state_key(&pkg_id);
                            let is_installed = state.packages.contains_key(&state_key);
                            // Add a marker for installed packages
                            if is_installed {
                                result.name = format!("{} ✓", result.name);
                            }
                        }

                        all_results.extend(results);
                    }
                    Err(e) => {
                        output::warning(&format!("Failed to search {}: {}", backend, e));
                    }
                }
            }
            Ok(_) => {
                output::info(&format!("Backend {} does not support search", backend));
            }
            Err(e) => {
                output::warning(&format!("Failed to initialize {}: {}", backend, e));
            }
        }
    }

    // Filter results based on options
    if options.installed_only {
        all_results.retain(|r| r.name.contains('✓'));
    }

    if options.available_only {
        all_results.retain(|r| !r.name.contains('✓'));
    }

    // Display results
    display_results(&all_results, &actual_query);

    Ok(())
}

fn get_backends_to_search(options: &SearchOptions) -> Result<Vec<Backend>> {
    use crate::config::settings::Settings;

    // CLI flag overrides everything (intentional design)
    if options.backends.is_some() {
        let backend_list = options.backends.as_ref().unwrap();
        return backend_list
            .iter()
            .map(|b| parse_backend(b))
            .collect();
    }

    // Otherwise, respect backend settings
    let settings = Settings::load()
        .map_err(|e| crate::error::DeclarchError::Other(format!("Failed to load settings: {}", e)))?;

    let backend_mode = settings.get("backend_mode")
        .unwrap_or(&"auto".to_string())
        .clone();

    match backend_mode.as_str() {
        "enabled-only" => {
            // Only search backends listed in settings
            let backend_list = settings.get("backends")
                .unwrap_or(&"aur".to_string())
                .clone();

            backend_list
                .split(',')
                .map(|b| b.trim())
                .filter(|b| !b.is_empty())
                .map(|b| parse_backend(b))
                .collect()
        }
        "auto" | _ => {
            // Auto mode: search all backends that support search
            // Currently only AUR has search implemented
            Ok(vec![Backend::Aur])
        }
    }
}

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

fn create_manager_for_backend(backend: &Backend) -> Result<Box<dyn PackageManager>> {
    use crate::packages::create_manager;

    let global_config = crate::config::types::GlobalConfig::default();

    create_manager(backend, &global_config, false).map_err(|e| {
        crate::error::DeclarchError::Other(format!("Failed to create manager for {}: {}", backend, e))
    })
}

fn display_results(results: &[PackageSearchResult], query: &str) {
    if results.is_empty() {
        output::info(&format!("No packages found matching '{}'", query.cyan()));
        return;
    }

    // Group by backend
    let mut by_backend: std::collections::HashMap<Backend, Vec<&PackageSearchResult>> =
        std::collections::HashMap::new();

    for result in results {
        by_backend
            .entry(result.backend.clone())
            .or_default()
            .push(result);
    }

    output::success(&format!(
        "Found {} package(s) matching '{}':\n",
        results.len(),
        query.cyan()
    ));

    // Display by backend
    for (backend, packages) in by_backend {
        println!("{}", format!("{}:", backend).bold().cyan());

        for pkg in packages {
            let name = pkg.name.green();
            let version = pkg.version
                .as_ref()
                .map(|v| v.dimmed().to_string())
                .unwrap_or_default();
            let description = pkg.description
                .as_ref()
                .map(|d| format!(" - {}", d))
                .unwrap_or_default();

            println!("  {} {}{}", name, version, description.dimmed());
        }
        println!();
    }
}
