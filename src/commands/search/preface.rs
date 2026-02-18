use crate::core::types::Backend;
use crate::packages::traits::PackageSearchResult;
use crate::state;
use crate::ui as output;
use colored::Colorize;
use std::collections::HashMap;

use super::SearchResultOut;
use super::matching::mark_installed;
use super::render::{display_backend_results, sorted_backend_keys};

pub(super) fn append_managed_machine_hits(
    include_managed_hits: bool,
    managed_hits: &HashMap<String, Vec<PackageSearchResult>>,
    total_found: &mut usize,
    has_results: &mut bool,
    machine_results: &mut Vec<SearchResultOut>,
) {
    if !include_managed_hits {
        return;
    }

    for backend in sorted_backend_keys(managed_hits) {
        if let Some(results) = managed_hits.get(&backend) {
            *total_found += results.len();
            for result in results {
                machine_results.push(SearchResultOut {
                    backend: backend.clone(),
                    name: result.name.clone(),
                    version: result.version.clone(),
                    description: None,
                    installed: true,
                });
            }
            *has_results = *has_results || !results.is_empty();
        }
    }
}

pub(super) fn render_human_managed_preface(
    include_managed_hits: bool,
    managed_hits: &mut HashMap<String, Vec<PackageSearchResult>>,
    state: &state::types::State,
    actual_query: &str,
    has_results: &mut bool,
) {
    println!();
    output::info(&format!(
        "Searching for '{}' (streaming results)...",
        actual_query.cyan()
    ));
    println!();

    if include_managed_hits {
        for backend in sorted_backend_keys(managed_hits) {
            if let Some(results) = managed_hits.remove(&backend)
                && !results.is_empty()
            {
                *has_results = true;
                let backend_display = Backend::from(format!("managed/{}", backend));
                let marked_results = mark_installed(results, state, true);
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
