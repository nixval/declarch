use crate::core::types::Backend;
use crate::error::Result;
use crate::packages::traits::{PackageManager, PackageSearchResult};

#[derive(Debug)]
pub(super) enum BackendResult {
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

pub(super) fn search_single_backend(
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

pub(super) fn create_manager_from_config(
    config: &crate::backends::config::BackendConfig,
) -> Result<Box<dyn PackageManager>> {
    use crate::backends::GenericManager;

    let backend = Backend::from(config.name.clone());
    Ok(Box::new(GenericManager::from_config(
        config.clone(),
        backend,
        false,
    )))
}
