//! Package search command
//!
//! Search for packages across configured backends with streaming results.
//! Results from faster backends are displayed immediately without waiting for slower ones.

use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::constants::SEARCH_BACKEND_TIMEOUT_SECS;
use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::{PackageManager, PackageSearchResult};
use crate::state;
use crate::ui as output;
use crate::utils::machine_output;
use crate::utils::sanitize::validate_search_query;
use colored::Colorize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

    let runtime_config = load_runtime_config_for_command("search command");

    let mut backend_configs = crate::backends::load_all_backends_unified()?;
    for (name, cfg) in &mut backend_configs {
        apply_runtime_backend_overrides(cfg, name, &runtime_config);
    }

    // Get backends to search
    let (backends_to_search, selection_warnings) =
        get_backends_to_search(&updated_options, &backend_configs, machine_mode)?;

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
            output::info("Run 'declarch init --backend <name>' to add a backend");
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
        // Check against state with normalization/alias support.
        for result in &mut results {
            if is_installed_result(result, state, local_mode) {
                result.name = format!("{} ✓", result.name);
            }
        }
    }
    results
}

fn is_installed_result(
    result: &PackageSearchResult,
    state: &state::types::State,
    local_mode: bool,
) -> bool {
    if local_mode {
        return true;
    }

    let exact_pkg = crate::core::types::PackageId {
        name: result.name.clone(),
        backend: result.backend.clone(),
    };
    let exact_key = crate::core::resolver::make_state_key(&exact_pkg);
    if state.packages.contains_key(&exact_key) {
        return true;
    }

    let normalized_name = normalize_package_name(&result.name);
    let result_backend_group = canonical_backend_group(result.backend.name());

    state.packages.values().any(|pkg| {
        normalize_package_name(&pkg.config_name) == normalized_name
            && canonical_backend_group(pkg.backend.name()) == result_backend_group
    })
}

fn normalize_package_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

fn canonical_backend_group(backend: &str) -> &str {
    match backend {
        "aur" | "yay" | "paru" | "pacman" => "arch",
        _ => backend,
    }
}

fn should_show_backend_error(error: &str, verbose: bool, local_mode: bool) -> bool {
    if verbose {
        return true;
    }
    if local_mode {
        return false;
    }
    !error.starts_with("Local list fallback failed:")
}

fn run_managed_installed_search(
    query: &str,
    state: &state::types::State,
    options: &SearchOptions,
    machine_mode: bool,
) -> Result<()> {
    let mut grouped = collect_managed_hits(query, state, options);
    let backends = sorted_backend_keys(&grouped);

    if machine_mode {
        let mut out_results = Vec::new();
        for backend in backends {
            if let Some(results) = grouped.get(&backend) {
                for result in results {
                    out_results.push(SearchResultOut {
                        backend: backend.clone(),
                        name: result.name.clone(),
                        version: result.version.clone(),
                        description: None,
                        installed: true,
                    });
                }
            }
        }
        let report = SearchReportOut {
            query: query.to_string(),
            local: true,
            requested_backends: options.backends.clone(),
            total_matches: out_results.len(),
            shown_results: out_results.len(),
            results: out_results,
        };
        machine_output::emit_v1(
            "search",
            report,
            Vec::new(),
            Vec::new(),
            options.format.as_deref().unwrap_or("json"),
        )?;
        return Ok(());
    }

    println!();
    output::info(&format!(
        "Searching managed installed packages for '{}'",
        query.cyan()
    ));
    println!();

    let mut total_found = 0usize;
    let limit = options.limit.or(Some(10));
    let mut has_results = false;

    for backend_name in backends {
        let mut results = grouped.remove(&backend_name).unwrap_or_default();
        total_found += results.len();
        let backend = Backend::from(backend_name.clone());
        let backend_total = results.len();

        if let Some(limit_value) = limit
            && results.len() > limit_value
        {
            results.truncate(limit_value);
        }

        let marked_results = mark_installed(results, state, true);
        if !marked_results.is_empty() {
            has_results = true;
            display_backend_results(&backend, &marked_results, backend_total, limit);
        }
    }

    println!();
    if has_results {
        if let Some(limit_value) = limit
            && total_found > limit_value
        {
            output::info(&format!(
                "Showing limited results. Use --limit 0 for all {} matches.",
                total_found
            ));
        }
    } else {
        output::info(&format!(
            "No managed installed packages found matching '{}'",
            query.cyan()
        ));
    }

    Ok(())
}

fn collect_managed_hits(
    query: &str,
    state: &state::types::State,
    options: &SearchOptions,
) -> HashMap<String, Vec<PackageSearchResult>> {
    let query_lower = query.to_lowercase();
    let requested_backends: Option<HashSet<String>> = options
        .backends
        .as_ref()
        .map(|v| v.iter().map(|b| b.to_lowercase()).collect());
    let mut grouped: HashMap<String, Vec<PackageSearchResult>> = HashMap::new();

    for pkg in state.packages.values() {
        if let Some(requested) = &requested_backends
            && !requested.contains(pkg.backend.name())
        {
            continue;
        }
        let search_name = if pkg.config_name.is_empty() {
            pkg.provides_name.as_str()
        } else {
            pkg.config_name.as_str()
        };
        if !search_name.to_lowercase().contains(&query_lower) {
            continue;
        }

        grouped
            .entry(pkg.backend.to_string())
            .or_default()
            .push(PackageSearchResult {
                name: search_name.to_string(),
                version: pkg.version.clone(),
                description: None,
                backend: pkg.backend.clone(),
            });
    }

    for results in grouped.values_mut() {
        results.sort_by(|a, b| a.name.cmp(&b.name));
    }

    grouped
}

fn sorted_backend_keys(grouped: &HashMap<String, Vec<PackageSearchResult>>) -> Vec<String> {
    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort();
    backends
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
    if let Some(_limit_val) = limit
        && total_found > results.len()
    {
        println!(
            "  {} (showing {} of {})",
            "Limited results".dimmed(),
            results.len(),
            total_found
        );
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

fn get_backends_to_search(
    options: &SearchOptions,
    backend_configs: &HashMap<String, crate::backends::config::BackendConfig>,
    machine_mode: bool,
) -> Result<(Vec<Backend>, Vec<String>)> {
    let (result, unknown, unsupported, os_mismatch) =
        select_backends_to_search(backend_configs, options.backends.as_ref(), options.local);
    let mut warnings = Vec::new();

    if !unknown.is_empty() {
        let msg = format!("Unknown backend(s): {}", unknown.join(", "));
        if machine_mode {
            warnings.push(msg);
        } else {
            output::warning(&msg);
        }
    }
    if !unsupported.is_empty() {
        let capability = if options.local {
            "local search/list support"
        } else {
            "search support"
        };
        let msg = format!(
            "Skipped backend(s) without {}: {}",
            capability,
            unsupported.join(", ")
        );
        if machine_mode {
            warnings.push(msg);
        } else {
            output::warning(&msg);
        }
    }
    if !os_mismatch.is_empty() {
        let msg = format!(
            "Skipped backend(s) that are not for this OS: {}",
            os_mismatch.join(", ")
        );
        if machine_mode {
            warnings.push(msg);
            warnings.push(
                "This is normal when one config is shared across different machines.".to_string(),
            );
        } else {
            output::warning(&msg);
            output::info("This is normal when one config is shared across different machines.");
        }
    }

    if result.is_empty() {
        let msg = if options.local {
            "No backends with local search/list support configured".to_string()
        } else {
            "No backends with search support configured".to_string()
        };
        if options.local {
            if machine_mode {
                warnings.push(msg);
            } else {
                output::warning("No backends with local search/list support configured");
            }
        } else if machine_mode {
            warnings.push(msg);
        } else {
            output::warning("No backends with search support configured");
        }
        if machine_mode {
            warnings.push("Run 'declarch init --backend <name>' to add a backend".to_string());
        } else {
            output::info("Run 'declarch init --backend <name>' to add a backend");
        }
    }

    Ok((result, warnings))
}

fn select_backends_to_search(
    all_backends: &HashMap<String, crate::backends::config::BackendConfig>,
    requested_backends: Option<&Vec<String>>,
    local_mode: bool,
) -> (Vec<Backend>, Vec<String>, Vec<String>, Vec<String>) {
    let supports_mode = |config: &crate::backends::config::BackendConfig| {
        if local_mode {
            config.search_local_cmd.is_some() || config.list_cmd.is_some()
        } else {
            config.search_cmd.is_some()
        }
    };

    let mut selected = Vec::new();
    let mut unknown = Vec::new();
    let mut unsupported = Vec::new();
    let mut os_mismatch = Vec::new();

    if let Some(requested) = requested_backends {
        for name in requested {
            match all_backends.get(name) {
                Some(config)
                    if crate::utils::platform::backend_supports_current_os(config)
                        && supports_mode(config) =>
                {
                    selected.push(Backend::from(name.as_str()))
                }
                Some(config) if !crate::utils::platform::backend_supports_current_os(config) => {
                    os_mismatch.push(name.clone())
                }
                Some(_) => unsupported.push(name.clone()),
                None => unknown.push(name.clone()),
            }
        }
    } else {
        let mut entries: Vec<_> = all_backends.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut selected_local_groups: HashSet<String> = HashSet::new();
        for (name, config) in entries {
            if crate::utils::platform::backend_supports_current_os(config) && supports_mode(config)
            {
                if local_mode {
                    let group = canonical_backend_group(name).to_string();
                    if selected_local_groups.contains(&group) {
                        continue;
                    }
                    selected_local_groups.insert(group);
                }
                selected.push(Backend::from(name.as_str()));
            }
        }
    }

    unknown.sort();
    unsupported.sort();
    os_mismatch.sort();

    (selected, unknown, unsupported, os_mismatch)
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
    use crate::state::types::{PackageState, State};
    use chrono::Utc;

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
        let (selected_remote, unknown_remote, unsupported_remote, os_mismatch_remote) =
            select_backends_to_search(&all, Some(&requested), false);
        let names_remote: Vec<_> = selected_remote
            .iter()
            .map(|b| b.name().to_string())
            .collect();
        assert_eq!(names_remote, vec!["paru".to_string()]);
        assert_eq!(unknown_remote, vec!["missing".to_string()]);
        assert_eq!(unsupported_remote, vec!["pip".to_string()]);
        assert!(os_mismatch_remote.is_empty());

        let (selected_local, unknown_local, unsupported_local, os_mismatch_local) =
            select_backends_to_search(&all, Some(&requested), true);
        let names_local: Vec<_> = selected_local
            .iter()
            .map(|b| b.name().to_string())
            .collect();
        assert_eq!(names_local, vec!["pip".to_string()]);
        assert_eq!(unknown_local, vec!["missing".to_string()]);
        assert_eq!(unsupported_local, vec!["paru".to_string()]);
        assert!(os_mismatch_local.is_empty());
    }

    #[test]
    fn select_backends_auto_mode_is_sorted() {
        let mut all = HashMap::new();
        all.insert(
            "zypper".to_string(),
            BackendConfig {
                name: "zypper".to_string(),
                search_cmd: Some("zypper se {query}".to_string()),
                ..Default::default()
            },
        );
        all.insert(
            "apt".to_string(),
            BackendConfig {
                name: "apt".to_string(),
                search_cmd: Some("apt search {query}".to_string()),
                ..Default::default()
            },
        );

        let (selected, unknown, unsupported, os_mismatch) =
            select_backends_to_search(&all, None, false);
        let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names, vec!["apt".to_string(), "zypper".to_string()]);
        assert!(unknown.is_empty());
        assert!(unsupported.is_empty());
        assert!(os_mismatch.is_empty());
    }

    #[test]
    fn select_backends_local_mode_accepts_list_fallback() {
        let mut all = HashMap::new();
        all.insert(
            "flatpak".to_string(),
            BackendConfig {
                name: "flatpak".to_string(),
                list_cmd: Some("flatpak list".to_string()),
                ..Default::default()
            },
        );

        let (selected, unknown, unsupported, os_mismatch) =
            select_backends_to_search(&all, None, true);
        let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names, vec!["flatpak".to_string()]);
        assert!(unknown.is_empty());
        assert!(unsupported.is_empty());
        assert!(os_mismatch.is_empty());
    }

    #[test]
    fn select_backends_local_mode_deduplicates_arch_family() {
        let mut all = HashMap::new();
        all.insert(
            "aur".to_string(),
            BackendConfig {
                name: "aur".to_string(),
                search_local_cmd: Some("aur-local {query}".to_string()),
                ..Default::default()
            },
        );
        all.insert(
            "yay".to_string(),
            BackendConfig {
                name: "yay".to_string(),
                search_local_cmd: Some("yay-local {query}".to_string()),
                ..Default::default()
            },
        );
        all.insert(
            "flatpak".to_string(),
            BackendConfig {
                name: "flatpak".to_string(),
                search_local_cmd: Some("flatpak search --columns=application {query}".to_string()),
                ..Default::default()
            },
        );

        let (selected, unknown, unsupported, os_mismatch) =
            select_backends_to_search(&all, None, true);
        let names: Vec<_> = selected.iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names, vec!["aur".to_string(), "flatpak".to_string()]);
        assert!(unknown.is_empty());
        assert!(unsupported.is_empty());
        assert!(os_mismatch.is_empty());
    }

    #[test]
    fn normalize_package_name_strips_repo_prefix() {
        assert_eq!(normalize_package_name("bat"), "bat");
        assert_eq!(normalize_package_name("extra/bat"), "bat");
    }

    #[test]
    fn canonical_backend_group_maps_arch_family() {
        assert_eq!(canonical_backend_group("aur"), "arch");
        assert_eq!(canonical_backend_group("paru"), "arch");
        assert_eq!(canonical_backend_group("yay"), "arch");
        assert_eq!(canonical_backend_group("pacman"), "arch");
        assert_eq!(canonical_backend_group("flatpak"), "flatpak");
    }

    #[test]
    fn fallback_local_errors_are_suppressed_without_verbose() {
        assert!(!should_show_backend_error(
            "Local list fallback failed: Package manager error: xyz",
            false,
            true
        ));
        assert!(!should_show_backend_error(
            "Local search failed: xyz",
            false,
            true
        ));
        assert!(should_show_backend_error(
            "Search failed: xyz",
            false,
            false
        ));
        assert!(should_show_backend_error(
            "Local list fallback failed: Package manager error: xyz",
            true,
            true
        ));
    }

    #[test]
    fn installed_match_handles_repo_prefix_and_backend_alias() {
        let mut state = State::default();
        state.packages.insert(
            "aur:bat".to_string(),
            PackageState {
                backend: Backend::from("aur"),
                config_name: "bat".to_string(),
                provides_name: "bat".to_string(),
                actual_package_name: None,
                installed_at: Utc::now(),
                version: Some("0.25.0".to_string()),
                install_reason: Some("declared".to_string()),
                source_module: None,
                last_seen_at: None,
                backend_meta: None,
            },
        );

        let result = PackageSearchResult {
            name: "extra/bat".to_string(),
            version: Some("0.25.0".to_string()),
            description: None,
            backend: Backend::from("pacman"),
        };

        assert!(is_installed_result(&result, &state, false));
    }

    #[test]
    fn collect_managed_hits_respects_backend_filter() {
        let mut state = State::default();
        state.packages.insert(
            "aur:bat".to_string(),
            PackageState {
                backend: Backend::from("aur"),
                config_name: "bat".to_string(),
                provides_name: "bat".to_string(),
                actual_package_name: None,
                installed_at: Utc::now(),
                version: Some("0.25.0".to_string()),
                install_reason: Some("declared".to_string()),
                source_module: None,
                last_seen_at: None,
                backend_meta: None,
            },
        );
        state.packages.insert(
            "brew:hello".to_string(),
            PackageState {
                backend: Backend::from("brew"),
                config_name: "hello".to_string(),
                provides_name: "hello".to_string(),
                actual_package_name: None,
                installed_at: Utc::now(),
                version: Some("2.12.2".to_string()),
                install_reason: Some("declared".to_string()),
                source_module: None,
                last_seen_at: None,
                backend_meta: None,
            },
        );

        let options = SearchOptions {
            query: "hello".to_string(),
            backends: Some(vec!["brew".to_string()]),
            limit: None,
            installed_only: false,
            available_only: false,
            local: false,
            verbose: false,
            format: None,
            output_version: None,
        };

        let grouped = collect_managed_hits("hello", &state, &options);
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key("brew"));
    }
}
