use crate::config::loader::MergedConfig;
use crate::core::types::{Backend, PackageId};
use std::collections::HashMap;

pub(super) fn find_matches(
    config: &MergedConfig,
    backend_filter: Option<&Backend>,
    needle: &str,
    exact: bool,
) -> Vec<PackageId> {
    let needle_lower = needle.to_lowercase();

    let mut matches: Vec<_> = config
        .packages
        .keys()
        .filter(|pkg| backend_filter.is_none_or(|b| pkg.backend == *b))
        .filter(|pkg| {
            if exact {
                pkg.name.eq_ignore_ascii_case(needle)
            } else {
                pkg.name.to_lowercase().contains(&needle_lower)
            }
        })
        .cloned()
        .collect();

    matches.sort_by_key(|a| a.to_string());
    matches
}

pub(super) fn load_known_backends(
    config: &MergedConfig,
) -> HashMap<String, crate::backends::config::BackendConfig> {
    let mut backends = crate::backends::load_all_backends_unified().unwrap_or_default();
    for backend in &config.backends {
        backends.insert(backend.name.to_lowercase(), backend.clone());
    }
    backends
}
