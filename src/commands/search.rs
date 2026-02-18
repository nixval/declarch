//! Package search command
//!
//! Search for packages across configured backends with streaming results.
//! Results from faster backends are displayed immediately without waiting for slower ones.

mod backend_runtime;
mod managed;
mod matching;
mod preface;
mod render;
mod reporting;
mod selection;
mod setup;

use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::constants::SEARCH_BACKEND_TIMEOUT_SECS;
#[cfg(test)]
use crate::core::types::Backend;
use crate::error::Result;
use crate::project_identity;
use crate::state;
use crate::ui as output;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use backend_runtime::{BackendResult, create_manager_from_config, search_single_backend};
use managed::{collect_managed_hits, run_managed_installed_search};
#[cfg(test)]
use matching::normalize_package_name;
use matching::{
    canonical_backend_group, is_installed_result, mark_installed, parse_backend_query,
    should_show_backend_error,
};
use preface::{append_managed_machine_hits, render_human_managed_preface};
use render::display_backend_results;
use reporting::{emit_machine_report, emit_no_backends_report, show_human_summary};
use selection::get_backends_to_search;
use setup::normalize_search_request;

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

pub fn run(options: SearchOptions) -> Result<()> {
    // Load state to check installed packages
    let state = state::io::load_state()?;

    let (updated_options, actual_query, machine_mode) = normalize_search_request(&options)?;

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
    log_backend_selection_verbose(
        &options,
        &updated_options,
        machine_mode,
        &backends_to_search,
    );

    if backends_to_search.is_empty() {
        if machine_mode {
            emit_no_backends_report(
                &actual_query,
                options.local,
                updated_options.backends.clone(),
                selection_warnings,
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
    append_managed_machine_hits(
        include_managed_hits,
        &managed_hits,
        &mut total_found,
        &mut has_results,
        &mut machine_results,
    );
    if !machine_mode {
        render_human_managed_preface(
            include_managed_hits,
            &mut managed_hits,
            &state,
            &actual_query,
            &mut has_results,
        );
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
        emit_machine_report(
            &actual_query,
            options.local,
            updated_options.backends.clone(),
            total_found,
            machine_results,
            machine_warnings,
            options.format.as_deref().unwrap_or("json"),
        )?;
    } else {
        show_human_summary(has_results, total_found, effective_limit, &actual_query);
    }

    Ok(())
}

fn log_backend_selection_verbose(
    options: &SearchOptions,
    updated_options: &SearchOptions,
    machine_mode: bool,
    backends_to_search: &[crate::core::types::Backend],
) {
    if !options.verbose || machine_mode {
        return;
    }

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

#[cfg(test)]
mod tests;
