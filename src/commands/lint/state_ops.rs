use super::{LintIssue, LintOptions};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use chrono::Utc;
use std::collections::{HashMap, HashSet};

pub(super) fn handle_state_remove(options: &LintOptions) -> Result<bool> {
    let has_state_remove =
        !options.state_rm.is_empty() || options.state_rm_backend.is_some() || options.state_rm_all;
    if !has_state_remove {
        return Ok(false);
    }

    if options.state_rm.is_empty() && !options.state_rm_all {
        return Err(DeclarchError::Other(
            "State remove requires package IDs via --state-rm, or --state-rm-backend with --state-rm-all."
                .to_string(),
        ));
    }

    if options.state_rm_all && !options.state_rm.is_empty() {
        return Err(DeclarchError::Other(
            "Do not combine --state-rm-all with --state-rm. Use one strategy at a time."
                .to_string(),
        ));
    }

    if options.state_rm_backend.is_some() && options.state_rm.iter().any(|id| id.contains(':')) {
        return Err(DeclarchError::Other(
            "Do not combine prefixed IDs (backend:name) with --state-rm-backend.".to_string(),
        ));
    }

    let lock = crate::state::io::acquire_lock().map_err(|e| {
        DeclarchError::Other(format!(
            "Cannot modify state now: {}\nIf no other {} process is running, delete the lock file manually.",
            e,
            project_identity::BINARY_NAME
        ))
    })?;

    let mut state = crate::state::io::load_state()?;
    let keys_to_remove = resolve_state_remove_keys(
        &state.packages,
        &options.state_rm,
        options.state_rm_backend.as_deref(),
        options.state_rm_all,
    )?;

    if keys_to_remove.is_empty() {
        output::warning("No matching state entries found.");
        return Ok(true);
    }

    output::header("State Remove Plan");
    output::keyval("Entries to remove", &keys_to_remove.len().to_string());
    for key in &keys_to_remove {
        output::indent(&format!("• {}", key), 1);
    }
    output::separator();

    if options.dry_run {
        output::info("Dry run completed - no state changes made.");
        return Ok(true);
    }

    if !options.yes
        && !output::prompt_yes_no(&format!(
            "Remove these entries from {} state only (no package uninstall)?",
            project_identity::BINARY_NAME
        ))
    {
        output::warning("State remove cancelled by user.");
        return Ok(true);
    }

    let mut removed = 0usize;
    for key in keys_to_remove {
        if state.packages.remove(&key).is_some() {
            removed += 1;
        }
    }
    state.meta.last_sync = Utc::now();
    crate::state::io::save_state_locked(&state, &lock)?;

    output::success(&format!("Removed {} state entries.", removed));
    Ok(true)
}

pub(super) fn resolve_state_remove_keys(
    packages: &HashMap<String, crate::state::types::PackageState>,
    ids: &[String],
    backend_filter: Option<&str>,
    remove_all_for_backend: bool,
) -> Result<Vec<String>> {
    let mut keys_to_remove: HashSet<String> = HashSet::new();
    let mut missing: Vec<String> = Vec::new();
    let mut ambiguous: Vec<(String, Vec<String>)> = Vec::new();
    let backend_filter = backend_filter.map(|b| b.to_lowercase());

    if remove_all_for_backend {
        let backend = backend_filter.ok_or_else(|| {
            DeclarchError::Other("--state-rm-all requires --state-rm-backend <BACKEND>.".into())
        })?;
        for key in packages.keys() {
            if key.starts_with(&(backend.clone() + ":")) {
                keys_to_remove.insert(key.clone());
            }
        }
    } else {
        for raw_id in ids {
            if let Some((backend, name)) = raw_id.split_once(':') {
                let key = format!("{}:{}", backend.to_lowercase(), name);
                if packages.contains_key(&key) {
                    keys_to_remove.insert(key);
                } else {
                    missing.push(raw_id.clone());
                }
                continue;
            }

            if let Some(backend) = &backend_filter {
                let key = format!("{}:{}", backend, raw_id);
                if packages.contains_key(&key) {
                    keys_to_remove.insert(key);
                } else {
                    missing.push(raw_id.clone());
                }
                continue;
            }

            let matches: Vec<String> = packages
                .keys()
                .filter(|key| key.split_once(':').map(|(_, n)| n) == Some(raw_id.as_str()))
                .cloned()
                .collect();
            match matches.len() {
                0 => missing.push(raw_id.clone()),
                1 => {
                    if let Some(only) = matches.into_iter().next() {
                        keys_to_remove.insert(only);
                    }
                }
                _ => ambiguous.push((raw_id.clone(), matches)),
            }
        }
    }

    if !ambiguous.is_empty() {
        let mut details = Vec::new();
        for (name, candidates) in ambiguous {
            details.push(format!(
                "{} matched multiple backends: {}. Use backend:name or --state-rm-backend.",
                name,
                candidates.join(", ")
            ));
        }
        return Err(DeclarchError::Other(details.join("\n")));
    }

    if !missing.is_empty() {
        output::warning(&format!(
            "Some IDs were not found in state and will be skipped: {}",
            missing.join(", ")
        ));
    }

    let mut keys: Vec<String> = keys_to_remove.into_iter().collect();
    keys.sort();
    Ok(keys)
}

pub(super) fn collect_state_issues(issues: &mut Vec<LintIssue>) -> Result<()> {
    let state = crate::state::io::load_state()?;
    for issue in crate::state::io::validate_state_integrity(&state) {
        issues.push(LintIssue::warning(None, format!("State issue: {}", issue)));
    }
    Ok(())
}
