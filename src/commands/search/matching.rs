use crate::core::resolver;
use crate::core::types::PackageId;
use crate::packages::traits::PackageSearchResult;
use crate::state;

/// Parse query for optional "backend:query" syntax.
pub(super) fn parse_backend_query(query: &str) -> (Option<String>, String) {
    if query.contains(':') {
        let parts: Vec<&str> = query.splitn(2, ':').collect();
        if parts.len() == 2 {
            let potential_backend = parts[0].trim();
            let actual_query = parts[1].trim();

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

/// Mark installed packages with checkmark.
pub(super) fn mark_installed(
    mut results: Vec<PackageSearchResult>,
    state: &state::types::State,
    local_mode: bool,
) -> Vec<PackageSearchResult> {
    if local_mode {
        for result in &mut results {
            if !result.name.contains('✓') {
                result.name = format!("{} ✓", result.name);
            }
        }
    } else {
        for result in &mut results {
            if is_installed_result(result, state, local_mode) {
                result.name = format!("{} ✓", result.name);
            }
        }
    }
    results
}

pub(super) fn is_installed_result(
    result: &PackageSearchResult,
    state: &state::types::State,
    local_mode: bool,
) -> bool {
    if local_mode {
        return true;
    }

    let exact_pkg = PackageId {
        name: result.name.clone(),
        backend: result.backend.clone(),
    };
    let exact_key = resolver::make_state_key(&exact_pkg);
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

pub(super) fn normalize_package_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

pub(super) fn canonical_backend_group(backend: &str) -> &str {
    match backend {
        "aur" | "yay" | "paru" | "pacman" => "arch",
        _ => backend,
    }
}

pub(super) fn should_show_backend_error(error: &str, verbose: bool, local_mode: bool) -> bool {
    if verbose {
        return true;
    }
    if local_mode {
        return false;
    }
    !error.starts_with("Local list fallback failed:")
}
