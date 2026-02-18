use crate::error::Result;
use crate::utils::sanitize::validate_search_query;

use super::{SearchOptions, parse_backend_query};

pub(super) fn normalize_search_request(
    options: &SearchOptions,
) -> Result<(SearchOptions, String, bool)> {
    let (backend_from_query, actual_query) = parse_backend_query(&options.query);
    validate_search_query(&actual_query)?;

    let final_backends = if let Some(backend) = backend_from_query {
        Some(vec![backend])
    } else {
        options.backends.clone()
    };

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

    Ok((updated_options, actual_query, machine_mode))
}
