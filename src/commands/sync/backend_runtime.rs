use super::{
    InstalledSnapshot, ManagerMap, SyncOptions, apply_backend_env_overrides,
    apply_backend_option_overrides, apply_backend_package_sources,
};
use crate::config::loader;
use crate::core::types::{PackageId, SyncTarget};
use crate::error::Result;
use crate::packages::PackageManager;
use crate::project_identity;
use crate::state::types::Backend;
use crate::ui as output;
use std::collections::HashMap;

pub(super) fn initialize_managers_and_snapshot(
    config: &loader::MergedConfig,
    options: &SyncOptions,
    sync_target: &SyncTarget,
) -> Result<(InstalledSnapshot, ManagerMap)> {
    let mut installed_snapshot: InstalledSnapshot = HashMap::new();
    let mut managers: ManagerMap = HashMap::new();

    let mut known_backends = crate::backends::load_all_backends_unified()?;
    for backend in &config.backends {
        known_backends.insert(backend.name.clone(), backend.clone());
    }

    let configured_backends: std::collections::HashSet<Backend> = config
        .packages
        .keys()
        .map(|pkg_id| pkg_id.backend.clone())
        .collect();

    for backend in configured_backends {
        let backend_name = backend.name().to_string();
        let Some(mut backend_config) = known_backends.get(&backend_name).cloned() else {
            output::warning(&format!(
                "Backend '{}' is referenced by packages but has no config. Run '{}'",
                backend_name,
                project_identity::cli_with(&format!("init --backend {}", backend_name))
            ));
            continue;
        };

        apply_backend_option_overrides(&mut backend_config, &backend_name, config);
        apply_backend_env_overrides(&mut backend_config, &backend_name, config);
        apply_backend_package_sources(&mut backend_config, &backend_name, config);

        if !crate::utils::platform::backend_supports_current_os(&backend_config) {
            let current_os = crate::utils::platform::current_os_tag();
            let supported = crate::utils::platform::supported_os_summary(&backend_config);
            output::warning(&format!(
                "Skipping backend '{}' on this device (current OS: {}, supported: {}).",
                backend_name, current_os, supported
            ));
            output::info("This is okay. Keep it in your config for other machines.");
            continue;
        }

        let manager: Box<dyn PackageManager> =
            Box::new(crate::backends::GenericManager::from_config(
                backend_config,
                backend.clone(),
                options.noconfirm,
            ));

        let available = manager.is_available();

        if !available && matches!(sync_target, SyncTarget::Backend(b) if b == &backend) {
            output::warning(&format!(
                "Backend '{}' is not available on this system.",
                backend
            ));
        }

        if available {
            match manager.list_installed() {
                Ok(packages) => {
                    for (name, meta) in packages {
                        let pkg_id = PackageId {
                            name: name.clone(),
                            backend: backend.clone(),
                        };
                        installed_snapshot.insert(pkg_id, meta);
                    }
                }
                Err(e) => {
                    output::warning(&format!("Failed to list packages for {}: {}", backend, e));
                }
            }
            managers.insert(backend.clone(), manager);
        }
    }

    Ok((installed_snapshot, managers))
}

pub(super) fn refresh_installed_snapshot(managers: &ManagerMap) -> InstalledSnapshot {
    let mut snapshot = InstalledSnapshot::new();
    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }
        match manager.list_installed() {
            Ok(packages) => {
                for (name, meta) in packages {
                    snapshot.insert(
                        PackageId {
                            name,
                            backend: backend.clone(),
                        },
                        meta,
                    );
                }
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to refresh package snapshot for {}: {}",
                    backend, e
                ));
            }
        }
    }
    snapshot
}

pub(super) fn execute_backend_updates(managers: &ManagerMap, verbose: bool) -> Result<()> {
    output::separator();
    output::info("Updating package indices...");

    let mut updated_count = 0;
    let mut skipped_count = 0;

    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }

        if !manager.supports_update() {
            if verbose {
                output::verbose(&format!("Skipping '{}': no update_cmd configured", backend));
            }
            skipped_count += 1;
            continue;
        }

        match manager.update() {
            Ok(()) => {
                updated_count += 1;
            }
            Err(e) => {
                output::warning(&format!("Failed to update '{}': {}", backend, e));
                skipped_count += 1;
            }
        }
    }

    if updated_count > 0 {
        output::info(&format!("Updated {} backend(s)", updated_count));
    }
    if skipped_count > 0 {
        output::info(&format!("Skipped {} backend(s)", skipped_count));
    }

    Ok(())
}
