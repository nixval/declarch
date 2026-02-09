//! Package search command
//!
//! Search for packages across configured backends

use crate::core::types::{Backend, PackageId};
use crate::error::Result;
use crate::packages::traits::{PackageManager, PackageSearchResult};
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
fn parse_backend_query(query: &str) -> (Option<String>, String) {
    if query.contains(':') {
        let parts: Vec<&str> = query.splitn(2, ':').collect();
        if parts.len() == 2 {
            let potential_backend = parts[0].trim();
            let actual_query = parts[1].trim();

            // Check if it looks like a backend name (no spaces, alphanumeric)
            if !potential_backend.contains(' ') && !potential_backend.is_empty() {
                return (
                    Some(potential_backend.to_string()),
                    actual_query.to_string(),
                );
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
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Search across backends and collect results
    let mut all_results: Vec<PackageSearchResult> = Vec::new();
    let mut total_count = 0;

    // Default limit is 10 if not specified
    let effective_limit = updated_options.limit.or(Some(10));

    for backend in backends_to_search {
        match create_manager_for_backend(&backend) {
            Ok(manager) if manager.supports_search() => {
                output::info(&format!("Searching {}...", backend));

                match manager.search(&actual_query) {
                    Ok(mut results) => {
                        total_count += results.len();

                        // Apply result limiting
                        if let Some(limit_value) = effective_limit
                            && results.len() > limit_value
                        {
                            results.truncate(limit_value);
                        }

                        // Mark installed packages
                        for result in &mut results {
                            let pkg_id = PackageId {
                                name: result.name.clone(),
                                backend: result.backend.clone(),
                            };
                            let state_key = crate::core::resolver::make_state_key(&pkg_id);
                            let is_installed = state.packages.contains_key(&state_key);
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
                // Backend doesn't support search
                output::warning(&format!(
                    "Backend '{}' does not support search",
                    backend
                ));
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
    display_results(&all_results, &actual_query, total_count, effective_limit);

    Ok(())
}

fn get_backends_to_search(options: &SearchOptions) -> Result<Vec<Backend>> {
    // CLI flag overrides everything
    if let Some(backend_list) = &options.backends {
        return Ok(backend_list.iter().map(|b| Backend::from(b.as_str())).collect());
    }

    // No backends specified - try to use configured backends
    // Load from backends.kdl and check which support search
    match crate::backends::load_all_backends() {
        Ok(backends) => {
            let mut result = Vec::new();
            for (name, config) in backends {
                // Check if backend has search configured
                if config.search_cmd.is_some() {
                    result.push(Backend::from(name));
                }
            }
            
            if result.is_empty() {
                output::warning("No backends with search support configured");
                output::info("Run 'declarch init --backend <name>' to add a backend");
            }
            
            Ok(result)
        }
        Err(e) => {
            output::warning(&format!("Could not load backends: {}", e));
            Ok(Vec::new())
        }
    }
}

fn create_manager_for_backend(backend: &Backend) -> Result<Box<dyn PackageManager>> {
    use crate::packages::create_manager;

    let global_config = crate::config::types::GlobalConfig::default();

    create_manager(backend, &global_config, false).map_err(|e| {
        crate::error::DeclarchError::Other(format!(
            "Failed to create manager for {}: {}",
            backend, e
        ))
    })
}

fn display_results(
    results: &[PackageSearchResult],
    query: &str,
    total_count: usize,
    limit: Option<usize>,
) {
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

    // Display count with limit note if applicable
    if let Some(limit_val) = limit {
        if total_count > results.len() {
            output::success(&format!(
                "Found {} packages matching '{}' --limit {} (showing {}):\n",
                total_count,
                query.cyan(),
                limit_val,
                results.len()
            ));
        } else {
            output::success(&format!(
                "Found {} package(s) matching '{}':\n",
                results.len(),
                query.cyan()
            ));
        }
    } else {
        output::success(&format!(
            "Found {} package(s) matching '{}':\n",
            results.len(),
            query.cyan()
        ));
    }

    // Sort backends alphabetically
    let mut backends: Vec<_> = by_backend.keys().cloned().collect();
    backends.sort_by(|a, b| a.name().cmp(b.name()));

    // Display by backend
    for backend in backends {
        if let Some(packages) = by_backend.get(&backend) {
            println!("{}", format!("{}:", backend).bold().cyan());

            for pkg in packages.iter() {
                let name = pkg.name.green();
                let version = pkg
                    .version
                    .as_ref()
                    .map(|v| v.dimmed().to_string())
                    .unwrap_or_default();
                let description = pkg
                    .description
                    .as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();

                println!("  {} {}{}", name, version, description.dimmed());
            }
            println!();
        }
    }
}
