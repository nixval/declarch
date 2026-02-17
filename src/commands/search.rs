//! Package search command
//!
//! Search for packages across configured backends with streaming results.
//! Results from faster backends are displayed immediately without waiting for slower ones.

mod managed;
mod matching;
mod render;
mod selection;

use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::constants::SEARCH_BACKEND_TIMEOUT_SECS;
use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::project_identity;
use crate::state;
use crate::ui as output;
use crate::utils::machine_output;
use crate::utils::sanitize::validate_search_query;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use managed::{collect_managed_hits, run_managed_installed_search};
#[cfg(test)]
use matching::normalize_package_name;
use matching::{
    canonical_backend_group, is_installed_result, mark_installed, parse_backend_query,
    should_show_backend_error,
};
use render::{display_backend_results, sorted_backend_keys};
use selection::get_backends_to_search;

/// Maximum time to wait for a backend to respond (seconds)
const BACKEND_TIMEOUT_SECONDS: u64 = SEARCH_BACKEND_TIMEOUT_SECS;
/// Local search should feel responsive even with slow/misconfigured backends.
const LOCAL_BACKEND_TIMEOUT_SECONDS: u64 = 8;

pub struct SearchOptions {
    pub query: String,
    pub backends: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub installed_only: bool,
    pub available_only: bool,
    pub local: bool,
    pub verbose: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct SearchResultOut {
    backend: String,
    name: String,
    version: Option<String>,
    description: Option<String>,
    installed: bool,
}

#[derive(Debug, Serialize)]
struct SearchReportOut {
    query: String,
    local: bool,
    requested_backends: Option<Vec<String>>,
    total_matches: usize,
    shown_results: usize,
    results: Vec<SearchResultOut>,
}

/// Result from a backend search
enum BackendResult {
    Success {
        backend: Backend,
        results: Vec<PackageSearchResult>,
        total_found: usize,
        duration_ms: u128,
    },
    Error {
        backend: Backend,
        error: String,
        duration_ms: u128,
    },
}

pub fn run(options: SearchOptions) -> Result<()> {
    // Load state to check installed packages
    let state = state::io::load_state()?;

    // Parse "backend:query" syntax
    let (backend_from_query, actual_query) = parse_backend_query(&options.query);
    validate_search_query(&actual_query)?;

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
        verbose: options.verbose,
        format: options.format.clone(),
        output_version: options.output_version.clone(),
    };
    let machine_mode = matches!(options.output_version.as_deref(), Some("v1"))
        && matches!(options.format.as_deref(), Some("json" | "yaml"));

    if updated_options.installed_only && !updated_options.local {
        return run_managed_installed_search(&actual_query, &state, &updated_options, machine_mode);
    }
    let include_managed_hits = !updated_options.local
        && !updated_options.installed_only
        && !updated_options.available_only;
    let mut managed_hits = if include_managed_hits {
        collect_managed_hits(&actual_query, &state, &updated_options)
    } else {
        HashMap::new()
    };
    if options.verbose && !machine_mode && include_managed_hits {
        let preloaded = managed_hits.values().map(|v| v.len()).sum::<usize>();
        output::verbose(&format!("Managed state preloaded matches: {}", preloaded));
    }

    let runtime_config = load_runtime_config_for_command("search command");

    let mut backend_configs = crate::backends::load_all_backends_unified()?;
    for (name, cfg) in &mut backend_configs {
        apply_runtime_backend_overrides(cfg, name, &runtime_config);
    }

    // Get backends to search
    let (backends_to_search, selection_warnings) =
        get_backends_to_search(&updated_options, &backend_configs, machine_mode)?;
    if options.verbose && !machine_mode {
        let mode = if options.local {
            "local installed search"
        } else {
            "repository search"
        };
        let requested = updated_options
            .backends
            .as_ref()
            .map(|b| b.join(", "))
            .unwrap_or_else(|| "(auto)".to_string());
        let selected = backends_to_search
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        output::verbose(&format!("Search mode: {}", mode));
        output::verbose(&format!("Requested backends: {}", requested));
        output::verbose(&format!(
            "Selected backends ({}): {}",
            backends_to_search.len(),
            if selected.is_empty() {
                "(none)".to_string()
            } else {
                selected
            }
        ));
    }

    if backends_to_search.is_empty() {
        if machine_mode {
            let report = SearchReportOut {
                query: actual_query.clone(),
                local: options.local,
                requested_backends: updated_options.backends.clone(),
                total_matches: 0,
                shown_results: 0,
                results: Vec::new(),
            };
            machine_output::emit_v1(
                "search",
                report,
                selection_warnings,
                Vec::new(),
                options.format.as_deref().unwrap_or("json"),
            )?;
        } else {
            output::warning("No backends available for search");
            output::info(&format!(
                "Run '{}' to add a backend",
                project_identity::cli_with("init --backend <name>")
            ));
        }
        return Ok(());
    }

    // Default limit is 10 if not specified
    let effective_limit = updated_options.limit.or(Some(10));

    // Create channel for streaming results
    let (tx, rx) = mpsc::channel::<BackendResult>();

    // Spawn a thread for each backend
    let query_clone = actual_query.clone();
    let local_mode = options.local;
    for backend in backends_to_search {
        let Some(backend_config) = backend_configs.get(backend.name()).cloned() else {
            output::warning(&format!(
                "Skipping '{}': backend configuration not found",
                backend
            ));
            continue;
        };
        let mut backend_config = backend_config;
        let prefer_list_for_local_search = backend_config.prefer_list_for_local_search;
        // Search is read-only; never require sudo here.
        // This avoids password prompts/timeouts in non-interactive flows.
        backend_config.needs_sudo = false;

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
            let started_at = std::time::Instant::now();
            let result = search_single_backend(
                manager,
                &query,
                local_mode,
                effective_limit,
                prefer_list_for_local_search,
            );
            let duration_ms = started_at.elapsed().as_millis();

            // Send result (ignore errors if receiver dropped)
            match result {
                Ok((results, total)) => {
                    let _ = tx.send(BackendResult::Success {
                        backend,
                        results,
                        total_found: total,
                        duration_ms,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackendResult::Error {
                        backend,
                        error: e,
                        duration_ms,
                    });
                }
            }
        });
    }

    // Drop original sender so channel closes when all threads done
    drop(tx);

    // Collect and display results as they arrive
    let mut total_found = 0;
    let mut has_results = false;
    let mut machine_results: Vec<SearchResultOut> = Vec::new();
    let mut machine_warnings = selection_warnings;
    if include_managed_hits {
        for backend in sorted_backend_keys(&managed_hits) {
            if let Some(results) = managed_hits.get(&backend) {
                total_found += results.len();
                if machine_mode {
                    for result in results {
                        machine_results.push(SearchResultOut {
                            backend: backend.clone(),
                            name: result.name.clone(),
                            version: result.version.clone(),
                            description: None,
                            installed: true,
                        });
                    }
                    has_results = has_results || !results.is_empty();
                }
            }
        }
    }

    // Print initial message
    if !machine_mode {
        println!();
        output::info(&format!(
            "Searching for '{}' (streaming results)...",
            actual_query.cyan()
        ));
        println!();
        if include_managed_hits {
            for backend in sorted_backend_keys(&managed_hits) {
                if let Some(results) = managed_hits.remove(&backend)
                    && !results.is_empty()
                {
                    has_results = true;
                    let backend_display = Backend::from(format!("managed/{}", backend));
                    let marked_results = mark_installed(results, &state, true);
                    display_backend_results(
                        &backend_display,
                        &marked_results,
                        marked_results.len(),
                        None,
                    );
                }
            }
        }
    }

    // Receive results with timeout
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(if options.local {
        LOCAL_BACKEND_TIMEOUT_SECONDS
    } else {
        BACKEND_TIMEOUT_SECONDS
    });

    while let Ok(result) = rx.recv_timeout(timeout) {
        match result {
            BackendResult::Success {
                backend,
                results,
                total_found: backend_total,
                duration_ms,
            } => {
                if options.verbose && !machine_mode {
                    output::info(&format!("{} completed in {} ms", backend, duration_ms));
                }
                total_found += backend_total;
                if machine_mode {
                    let mut shown_for_backend = 0usize;
                    for result in results {
                        let installed = is_installed_result(&result, &state, local_mode);
                        if options.installed_only && !installed {
                            continue;
                        }
                        if options.available_only && installed {
                            continue;
                        }
                        shown_for_backend += 1;
                        machine_results.push(SearchResultOut {
                            backend: backend.to_string(),
                            name: result.name,
                            version: result.version,
                            description: result.description,
                            installed,
                        });
                    }
                    if shown_for_backend > 0 {
                        has_results = true;
                    }
                } else {
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
                        display_backend_results(
                            &backend,
                            &marked_results,
                            backend_total,
                            effective_limit,
                        );
                    }
                }
            }
            BackendResult::Error {
                backend,
                error,
                duration_ms,
            } => {
                if options.verbose && !machine_mode {
                    output::warning(&format!("{} failed after {} ms", backend, duration_ms));
                }
                if machine_mode {
                    machine_warnings.push(format!("{}: {}", backend, error));
                } else if should_show_backend_error(&error, options.verbose, options.local) {
                    output::warning(&format!("{}: {}", backend, error));
                }
            }
        }

        // Reset timeout for next receive
        let elapsed = start_time.elapsed();
        if elapsed > timeout {
            break;
        }
    }

    if options.verbose && !machine_mode && options.local && start_time.elapsed() >= timeout {
        output::info(&format!(
            "Stopped waiting after {}s; use --backends to target specific local sources.",
            timeout.as_secs()
        ));
    }

    if machine_mode {
        let report = SearchReportOut {
            query: actual_query.clone(),
            local: options.local,
            requested_backends: updated_options.backends.clone(),
            total_matches: total_found,
            shown_results: machine_results.len(),
            results: machine_results,
        };
        machine_output::emit_v1(
            "search",
            report,
            machine_warnings,
            Vec::new(),
            options.format.as_deref().unwrap_or("json"),
        )?;
    } else {
        // Summary
        println!();
        if has_results {
            if let Some(limit) = effective_limit
                && total_found > limit
            {
                output::info(&format!(
                    "Showing limited results. Use --limit 0 for all {} matches.",
                    total_found
                ));
            }
        } else {
            output::info(&format!(
                "No packages found matching '{}'",
                actual_query.cyan()
            ));
        }
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
    prefer_list_for_local_search: bool,
) -> std::result::Result<(Vec<PackageSearchResult>, usize), String> {
    if local_mode {
        let backend_name = manager.backend_type().name().to_string();

        let mut results = if manager.supports_search_local() && !prefer_list_for_local_search {
            manager
                .search_local(query)
                .map_err(|e| format!("Local search failed: {}", e))?
        } else {
            // Fallback for backends without dedicated search_local:
            // list installed packages and filter by query.
            let installed = manager
                .list_installed()
                .map_err(|e| format!("Local list fallback failed: {}", e))?;
            let query_lower = query.to_lowercase();
            installed
                .into_iter()
                .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
                .map(|(name, meta)| PackageSearchResult {
                    name,
                    version: meta.version,
                    description: None,
                    backend: Backend::from(backend_name.clone()),
                })
                .collect()
        };

        let total = results.len();
        if let Some(limit_value) = limit
            && results.len() > limit_value
        {
            results.truncate(limit_value);
        }
        Ok((results, total))
    } else {
        if !manager.supports_search() {
            return Err("Does not support search".to_string());
        }

        match manager.search(query) {
            Ok(mut results) => {
                let total = results.len();
                // Apply limit
                if let Some(limit_value) = limit
                    && results.len() > limit_value
                {
                    results.truncate(limit_value);
                }
                Ok((results, total))
            }
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }
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
mod tests;
