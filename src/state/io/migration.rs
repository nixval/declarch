use crate::error::Result;
use crate::project_identity;
use crate::state::types::{PackageState, State};

use super::{CURRENT_STATE_SCHEMA_VERSION, StateRepairReport};

pub(super) fn sanitize_state_in_place(state: &mut State) -> StateRepairReport {
    use crate::core::resolver;
    use crate::core::types::PackageId;
    use std::collections::{HashMap, HashSet};

    let total_before = state.packages.len();
    let mut report = StateRepairReport {
        total_before,
        ..Default::default()
    };

    let mut seen_signatures: HashSet<String> = HashSet::new();
    let mut repaired: HashMap<String, PackageState> = HashMap::new();

    for (old_key, pkg_state) in &state.packages {
        if pkg_state.config_name.trim().is_empty() {
            report.removed_empty_name += 1;
            continue;
        }

        let signature = format!("{}:{}", pkg_state.backend, pkg_state.config_name);
        if !seen_signatures.insert(signature) {
            report.removed_duplicates += 1;
            continue;
        }

        let mut normalized = pkg_state.clone();
        if normalized.provides_name.trim().is_empty() {
            normalized.provides_name = normalized.config_name.clone();
            report.normalized_fields += 1;
        }

        let canonical_key = resolver::make_state_key(&PackageId {
            name: normalized.config_name.clone(),
            backend: normalized.backend.clone(),
        });
        if &canonical_key != old_key {
            report.rekeyed_entries += 1;
        }
        repaired.insert(canonical_key, normalized);
    }

    report.total_after = repaired.len();
    state.packages = repaired;
    report
}

/// Migrate state to fix duplicate keys and format issues.
/// Returns true if migration was performed.
pub(super) fn migrate_state(state: &mut State) -> Result<bool> {
    use crate::core::resolver;
    use std::collections::HashMap;

    let mut migrated = false;
    let mut new_packages: HashMap<String, PackageState> = HashMap::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for pkg_state in state.packages.values() {
        let canonical_id = crate::core::types::PackageId {
            name: pkg_state.config_name.clone(),
            backend: pkg_state.backend.clone(),
        };
        let canonical_key = resolver::make_state_key(&canonical_id);

        let signature = format!("{}:{}", pkg_state.backend, pkg_state.config_name);
        if seen.contains(&signature) {
            migrated = true;
            continue;
        }

        seen.insert(signature);
        new_packages.insert(canonical_key, pkg_state.clone());
    }

    if migrate_state_schema(state) {
        migrated = true;
    }

    if migrated {
        state.packages = new_packages;
    }

    Ok(migrated)
}

fn migrate_state_schema(state: &mut State) -> bool {
    let mut changed = false;

    if state.meta.schema_version < CURRENT_STATE_SCHEMA_VERSION {
        state.meta.schema_version = CURRENT_STATE_SCHEMA_VERSION;
        changed = true;
    }

    if state.meta.state_revision.is_none() {
        state.meta.state_revision = Some(1);
        changed = true;
    }

    if state.meta.generator.is_none() {
        state.meta.generator = Some(project_identity::STABLE_PROJECT_ID.to_string());
        changed = true;
    }

    changed
}
