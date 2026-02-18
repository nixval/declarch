use crate::core::types::{Backend, PackageMetadata};
use crate::error::Result;
use crate::packages::PackageManager;
use crate::project_identity;
use crate::state::{self, io::StateLock, types::PackageState};
use crate::ui as output;
use chrono::Utc;
use std::collections::HashMap;

pub(super) fn persist_successful_transition(
    state: &mut state::types::State,
    lock: &StateLock,
    backend: &Backend,
    old_state_key: &str,
    new_package: &str,
    manager: &dyn PackageManager,
    installed_before: &HashMap<String, PackageMetadata>,
) -> Result<()> {
    output::info(&format!(
        "Updating {} state...",
        project_identity::BINARY_NAME
    ));

    state.packages.remove(old_state_key);

    let new_state_key = format!("{}:{}", backend, new_package);
    let post_transition_installed = manager.list_installed().ok();

    let new_pkg_state = PackageState {
        backend: backend.clone(),
        config_name: new_package.to_string(),
        provides_name: new_package.to_string(),
        actual_package_name: None,
        installed_at: Utc::now(),
        version: post_transition_installed
            .as_ref()
            .and_then(|pkgs| pkgs.get(new_package))
            .or_else(|| installed_before.get(new_package))
            .and_then(|m| m.version.clone()),
        install_reason: Some("manual-sync".to_string()),
        source_module: None,
        last_seen_at: Some(Utc::now()),
        backend_meta: None,
    };

    state.packages.insert(new_state_key, new_pkg_state);
    state.meta.last_sync = Utc::now();
    state::io::save_state_locked(state, lock)?;

    Ok(())
}
