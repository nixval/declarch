use super::{SearchReportOut, SearchResultOut};
use crate::error::Result;
use crate::ui as output;
use crate::utils::machine_output;
use colored::Colorize;

pub(super) fn emit_no_backends_report(
    query: &str,
    local: bool,
    requested_backends: Option<Vec<String>>,
    warnings: Vec<String>,
    format: &str,
) -> Result<()> {
    let report = SearchReportOut {
        query: query.to_string(),
        local,
        requested_backends,
        total_matches: 0,
        shown_results: 0,
        results: Vec::new(),
    };
    machine_output::emit_v1("search", report, warnings, Vec::new(), format)
}

pub(super) fn emit_machine_report(
    query: &str,
    local: bool,
    requested_backends: Option<Vec<String>>,
    total_matches: usize,
    results: Vec<SearchResultOut>,
    warnings: Vec<String>,
    format: &str,
) -> Result<()> {
    let report = SearchReportOut {
        query: query.to_string(),
        local,
        requested_backends,
        total_matches,
        shown_results: results.len(),
        results,
    };
    machine_output::emit_v1("search", report, warnings, Vec::new(), format)
}

pub(super) fn show_human_summary(
    has_results: bool,
    total_found: usize,
    effective_limit: Option<usize>,
    query: &str,
) {
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
        output::info(&format!("No packages found matching '{}'", query.cyan()));
    }
}
