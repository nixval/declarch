use super::{SearchOptions, SearchReportOut, SearchResultOut};
use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::PackageSearchResult;
use crate::state;
use crate::ui as output;
use crate::utils::machine_output;
use colored::Colorize;
use std::collections::{HashMap, HashSet};

use super::matching::mark_installed;
use super::render::{display_backend_results, sorted_backend_keys};

pub(super) fn run_managed_installed_search(
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
    if options.verbose {
        let backend_filter = options
            .backends
            .as_ref()
            .map(|b| b.join(", "))
            .unwrap_or_else(|| "(all managed backends)".to_string());
        output::verbose(&format!(
            "Managed packages tracked: {}",
            state.packages.len()
        ));
        output::verbose(&format!("Backend filter: {}", backend_filter));
    }

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

pub(super) fn collect_managed_hits(
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
