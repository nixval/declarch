use crate::core::types::Backend;
use crate::packages::traits::PackageSearchResult;
use colored::Colorize;
use std::collections::HashMap;

pub(super) fn sorted_backend_keys(
    grouped: &HashMap<String, Vec<PackageSearchResult>>,
) -> Vec<String> {
    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort();
    backends
}

/// Display results for a single backend immediately
pub(super) fn display_backend_results(
    backend: &Backend,
    results: &[PackageSearchResult],
    total_found: usize,
    limit: Option<usize>,
) {
    println!("{}", format!("{}:", backend).cyan().bold());

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

    for result in results {
        print_search_result(result);
    }

    println!();
}

/// Print a single search result
pub(super) fn print_search_result(result: &PackageSearchResult) {
    let name_colored = if result.name.contains('✓') {
        result.name.green()
    } else {
        result.name.cyan()
    };

    if let Some(ref desc) = result.description {
        println!("  {} - {}", name_colored, desc.as_str().dimmed());
    } else {
        println!("  {}", name_colored);
    }
}
