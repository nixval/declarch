use crate::core::types::{Backend, PackageId, PackageMetadata};
use crate::error::Result;
use crate::packages::PackageManager;
use crate::ui as output;
use rayon::prelude::*;

use super::{InstalledSnapshot, ManagerMap};

pub(super) fn build_installed_snapshot(managers: &ManagerMap) -> Result<InstalledSnapshot> {
    let backend_results: Vec<Vec<(PackageId, PackageMetadata)>> = if managers.len() <= 1 {
        managers
            .iter()
            .filter_map(|(backend, mgr)| list_installed_for_backend(backend, mgr.as_ref()))
            .collect::<Result<Vec<_>>>()?
    } else {
        managers
            .par_iter()
            .filter_map(|(backend, mgr)| list_installed_for_backend(backend, mgr.as_ref()))
            .collect::<Result<Vec<_>>>()?
    };

    Ok(backend_results.into_iter().flatten().collect())
}

fn list_installed_for_backend(
    backend: &Backend,
    mgr: &dyn PackageManager,
) -> Option<Result<Vec<(PackageId, PackageMetadata)>>> {
    if !mgr.is_available() {
        return None;
    }
    match mgr.list_installed() {
        Ok(packages) => {
            let packages_with_backend: Vec<_> = packages
                .into_iter()
                .map(|(name, meta)| {
                    let id = PackageId {
                        name,
                        backend: backend.clone(),
                    };
                    (id, meta)
                })
                .collect();
            Some(Ok(packages_with_backend))
        }
        Err(e) => {
            output::warning(&format!("Failed to list packages for {}: {}", backend, e));
            Some(Err(e))
        }
    }
}
