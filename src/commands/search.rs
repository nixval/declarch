//! Package search command
//!
//! Search for packages across configured backends with streaming results.
//! Results from faster backends are displayed immediately without waiting for slower ones.

use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::state;
use crate::ui as output;
use colored::Colorize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Maximum time to wait for a backend to respond (seconds)
const BACKEND_TIMEOUT_SECONDS: u64 = 30;

pub struct SearchOptions {
    pub query: String,
    pub backends: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub installed_only: bool,
    pub available_only: bool,
    pub local: bool,
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

/// Result from a backend search
enum BackendResult {
    Success {
        backend: Backend,
        results: Vec<PackageSearchResult>,
        total_found: usize,
    },
    Error {
        backend: Backend,
        error: String,
    },
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
        local: options.local,
    };

    // Get backends to search
    let backends_to_search = get_backends_to_search(&updated_options)?;

    if backends_to_search.is_empty() {
        output::warning("No backends available for search");
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Default limit is 10 if not specified
    let effective_limit = updated_options.limit.or(Some(10));

    // Create channel for streaming results
    let (tx, rx) = mpsc::channel::<BackendResult>();

    // Spawn a thread for each backend
    let query_clone = actual_query.clone();
    let local_mode = options.local;
    let backend_configs = crate::backends::load_all_backends_unified()?;

    for backend in backends_to_search {
        let Some(backend_config) = backend_configs.get(backend.name()).cloned() else {
            output::warning(&format!(
                "Skipping '{}': backend configuration not found",
                backend
            ));
            continue;
        };

        let manager = match create_manager_from_config(&backend_config) {
            Ok(m) => m,
            Err(e) => {
                output::warning(&format!("Skipping '{}': {}", backend, e));
                continue;
            }
        };

        let tx = tx.clone();
        let query = query_clone.clone();

        thread::spawn(move || {
            let result = search_single_backend(manager, &query, local_mode, effective_limit);
            
            // Send result (ignore errors if receiver dropped)
            match result {
                Ok((results, total)) => {
                    let _ = tx.send(BackendResult::Success {
                        backend,
                        results,
                        total_found: total,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackendResult::Error {
                        backend,
                        error: e,
                    });
                }
            }
        });
    }

    // Drop original sender so channel closes when all threads done
    drop(tx);

    // Collect and display results as they arrive
    let mut displayed_backends: std::collections::HashSet<Backend> = std::collections::HashSet::new();
    let mut total_found = 0;
    let mut has_results = false;

    // Print initial message
    println!();
    output::info(&format!(
        "Searching for '{}' (streaming results)...",
        actual_query.cyan()
    ));
    println!();

    // Receive results with timeout
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(BACKEND_TIMEOUT_SECONDS);

    while let Ok(result) = rx.recv_timeout(timeout) {
        match result {
            BackendResult::Success { backend, results, total_found: backend_total } => {
                total_found += backend_total;

                // Mark installed packages
                let mut marked_results = mark_installed(results, &state, local_mode);

                // Filter results
                if options.installed_only {
                    marked_results.retain(|r| r.name.contains('✓'));
                }
                if options.available_only {
                    marked_results.retain(|r| !r.name.contains('✓'));
                }

                if !marked_results.is_empty() {
                    has_results = true;
                    
                    // Display this backend's results immediately
                    display_backend_results(&backend, &marked_results, backend_total, effective_limit);
                    displayed_backends.insert(backend);
                }
            }
            BackendResult::Error { backend, error } => {
                output::warning(&format!("{}: {}", backend, error));
            }
        }

        // Reset timeout for next receive
        let elapsed = start_time.elapsed();
        if elapsed > timeout {
            break;
        }
    }

    // Summary
    println!();
    if has_results {
        if let Some(limit) = effective_limit {
            if total_found > limit {
                output::info(&format!(
                    "Showing limited results. Use --limit 0 for all {} matches.",
                    total_found
                ));
            }
        }
    } else {
        output::info(&format!("No packages found matching '{}'", actual_query.cyan()));
    }

    Ok(())
}

/// Search a single backend
/// Returns a std::result::Result (not crate::error::Result) since this runs in a thread
fn search_single_backend(
    manager: Box<dyn PackageManager>,
    query: &str,
    local_mode: bool,
    limit: Option<usize>,
) -> std::result::Result<(Vec<PackageSearchResult>, usize), String> {
    if local_mode {
        if !manager.supports_search_local() {
            return Err("Does not support local search".to_string());
        }

        match manager.search_local(query) {
            Ok(mut results) => {
                let total = results.len();
                // Apply limit
                if let Some(limit_value) = limit {
                    if results.len() > limit_value {
                        results.truncate(limit_value);
                    }
                }
                Ok((results, total))
            }
            Err(e) => Err(format!("Local search failed: {}", e)),
        }
    } else {
        if !manager.supports_search() {
            return Err("Does not support search".to_string());
        }

        match manager.search(query) {
            Ok(mut results) => {
                let total = results.len();
                // Apply limit
                if let Some(limit_value) = limit {
                    if results.len() > limit_value {
                        results.truncate(limit_value);
                    }
                }
                Ok((results, total))
            }
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }
}

/// Mark installed packages with checkmark
fn mark_installed(
    mut results: Vec<PackageSearchResult>,
    state: &state::types::State,
    local_mode: bool,
) -> Vec<PackageSearchResult> {
    if local_mode {
        // For local search, all results are installed
        for result in &mut results {
            if !result.name.contains('✓') {
                result.name = format!("{} ✓", result.name);
            }
        }
    } else {
        // Check against state
        for result in &mut results {
            let pkg_id = crate::core::types::PackageId {
                name: result.name.clone(),
                backend: result.backend.clone(),
            };
            let state_key = crate::core::resolver::make_state_key(&pkg_id);
            if state.packages.contains_key(&state_key) {
                result.name = format!("{} ✓", result.name);
            }
        }
    }
    results
}

/// Display results for a single backend immediately
fn display_backend_results(
    backend: &Backend,
    results: &[PackageSearchResult],
    total_found: usize,
    limit: Option<usize>,
) {
    // Print backend header
    println!("{}", format!("{}:", backend).cyan().bold());

    // Show limit note if applicable
    if let Some(_limit_val) = limit {
        if total_found > results.len() {
            println!(
                "  {} (showing {} of {})",
                "Limited results".dimmed(),
                results.len(),
                total_found
            );
        }
    }

    // Print results
    for result in results {
        print_search_result(result);
    }

    println!(); // Empty line between backends
}

/// Print a single search result
fn print_search_result(result: &PackageSearchResult) {
    let name_colored = if result.name.contains('✓') {
        result.name.green()
    } else {
        result.name.cyan()
    };

    if let Some(ref desc) = result.description {
        // Multi-line format for results with description
        println!("  {} - {}", name_colored, desc.as_str().dimmed());
    } else {
        // Single line format
        println!("  {}", name_colored);
    }
}

fn get_backends_to_search(options: &SearchOptions) -> Result<Vec<Backend>> {
    // Load backends from unified source (explicit imports + legacy fallback)
    let backends_to_use = crate::backends::load_all_backends_unified()?;
    let (result, unknown, unsupported) = select_backends_to_search(
        &backends_to_use,
        options.backends.as_ref(),
        options.local,
    );

    if !unknown.is_empty() {
        output::warning(&format!("Unknown backend(s): {}", unknown.join(", ")));
    }
    if !unsupported.is_empty() {
        let capability = if options.local {
            "local search support"
        } else {
            "search support"
        };
        output::warning(&format!(
            "Skipped backend(s) without {}: {}",
            capability,
            unsupported.join(", ")
        ));
    }

    if result.is_empty() {
        if options.local {
            output::warning("No backends with local search support configured");
        } else {
            output::warning("No backends with search support configured");
        }
        output::info("Run 'declarch init --backend <name>' to add a backend");
    }

    Ok(result)
}

fn select_backends_to_search(
    all_backends: &HashMap<String, crate::backends::config::BackendConfig>,
    requested_backends: Option<&Vec<String>>,
    local_mode: bool,
) -> (Vec<Backend>, Vec<String>, Vec<String>) {
    let supports_mode = |config: &crate::backends::config::BackendConfig| {
        if local_mode {
            config.search_local_cmd.is_some()
        } else {
            config.search_cmd.is_some()
        }
    };

    let mut selected = Vec::new();
    let mut unknown = Vec::new();
    let mut unsupported = Vec::new();

    if let Some(requested) = requested_backends {
        for name in requested {
            match all_backends.get(name) {
                Some(config) if supports_mode(config) => selected.push(Backend::from(name.as_str())),
                Some(_) => unsupported.push(name.clone()),
                None => unknown.push(name.clone()),
            }
        }
    } else {
        for (name, config) in all_backends {
            if supports_mode(config) {
                selected.push(Backend::from(name.as_str()));
            }
        }
    }

    (selected, unknown, unsupported)
}

/// Create manager from backend config directly (for import-based architecture)
fn create_manager_from_config(
    config: &crate::backends::config::BackendConfig,
) -> Result<Box<dyn PackageManager>> {
    use crate::backends::GenericManager;
    use crate::core::types::Backend;
    
    let backend = Backend::from(config.name.clone());
    Ok(Box::new(GenericManager::from_config(
        config.clone(),
        backend,
        false,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::BackendConfig;

    #[test]
    fn select_backends_filters_unknown_and_unsupported() {
        let mut all = HashMap::new();
        all.insert(
            "paru".to_string(),
            BackendConfig {
                name: "paru".to_string(),
                search_cmd: Some("paru -Ss {query}".to_string()),
                ..Default::default()
            },
        );
        all.insert(
            "pip".to_string(),
            BackendConfig {
                name: "pip".to_string(),
                search_local_cmd: Some("pip list | grep {query}".to_string()),
                ..Default::default()
            },
        );

        let requested = vec!["paru".to_string(), "pip".to_string(), "missing".to_string()];
        let (selected_remote, unknown_remote, unsupported_remote) =
            select_backends_to_search(&all, Some(&requested), false);
        let names_remote: Vec<_> = selected_remote.iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names_remote, vec!["paru".to_string()]);
        assert_eq!(unknown_remote, vec!["missing".to_string()]);
        assert_eq!(unsupported_remote, vec!["pip".to_string()]);

        let (selected_local, unknown_local, unsupported_local) =
            select_backends_to_search(&all, Some(&requested), true);
        let names_local: Vec<_> = selected_local.iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names_local, vec!["pip".to_string()]);
        assert_eq!(unknown_local, vec!["missing".to_string()]);
        assert_eq!(unsupported_local, vec!["paru".to_string()]);
    }
}
