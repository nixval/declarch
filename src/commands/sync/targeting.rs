use crate::config::loader;
use crate::core::types::SyncTarget;
use crate::state::types::Backend;

pub(super) fn resolve_target(target: &Option<String>, config: &loader::MergedConfig) -> SyncTarget {
    if let Some(t) = target {
        let normalized_backend = Backend::from(t.as_str());
        let matches_backend_in_packages = config
            .packages
            .keys()
            .any(|pkg_id| pkg_id.backend == normalized_backend);
        let matches_backend_in_imports = config
            .backends
            .iter()
            .any(|backend| backend.name.eq_ignore_ascii_case(t));

        if matches_backend_in_packages || matches_backend_in_imports {
            SyncTarget::Backend(normalized_backend)
        } else {
            SyncTarget::Named(t.clone())
        }
    } else {
        SyncTarget::All
    }
}

pub(super) fn named_target_exists(config: &loader::MergedConfig, query: &str) -> bool {
    let query_lower = query.to_lowercase();

    for (pkg_id, sources) in &config.packages {
        if pkg_id.name == query {
            return true;
        }

        for source in sources {
            if let Some(stem) = source.file_stem()
                && stem.to_string_lossy().to_lowercase() == query_lower
            {
                return true;
            }
        }
    }

    false
}
