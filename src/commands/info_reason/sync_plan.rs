use crate::config::loader::MergedConfig;
use crate::core::types::PackageId;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use std::collections::HashSet;

use super::presentation::render_install_remove_reason_block;

pub(super) fn explain_sync_plan(config: &MergedConfig) -> Result<()> {
    output::header("Info: plan");

    let state = state::io::load_state()?;

    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
    let mut state_set: HashSet<PackageId> = HashSet::new();

    for pkg_state in state.packages.values() {
        state_set.insert(PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        });
    }

    let to_install: Vec<_> = config_set.difference(&state_set).cloned().collect();
    let to_remove: Vec<_> = state_set.difference(&config_set).cloned().collect();

    output::keyval("Declared packages", &config.packages.len().to_string());
    output::keyval("State packages", &state.packages.len().to_string());
    output::keyval("Planned install", &to_install.len().to_string());
    output::keyval("Planned remove", &to_remove.len().to_string());

    let duplicates = config.get_duplicates();
    if !duplicates.is_empty() {
        output::warning(&format!(
            "Found {} duplicate declarations across modules",
            duplicates.len()
        ));
    }

    let conflicts = config.get_cross_backend_conflicts();
    if !conflicts.is_empty() {
        output::warning(&format!(
            "Found {} cross-backend package name conflicts",
            conflicts.len()
        ));
    }

    if to_install.is_empty() && to_remove.is_empty() {
        output::success("No install/remove drift detected");
        return Ok(());
    }

    render_install_remove_reason_block("installed", '+', &to_install);
    render_install_remove_reason_block("removed", '-', &to_remove);

    Ok(())
}
