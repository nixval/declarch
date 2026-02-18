//! Transaction planning for sync operations
//!
//! Determines what packages to install, adopt, prune, and update.

mod filtering;
mod presentation;
mod variant_transition;
mod warnings;

use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::config::loader;
use crate::core::{resolver, types::SyncTarget};
use crate::error::Result;
use crate::state::types::State;
use filtering::resolve_filtered_transaction;
use presentation::{display_dry_run_details_impl, display_transaction_plan_impl};
use variant_transition::{collect_variant_mismatches, emit_variant_transition_error};
use warnings::warn_partial_upgrade_impl;

/// Create transaction from current state and desired config
/// This is a wrapper that calls resolve_and_filter_packages
pub fn create_transaction(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    resolve_and_filter_packages(config, state, installed_snapshot, managers, sync_target)
}

/// Resolve transaction and filter packages by available backends
pub fn resolve_and_filter_packages(
    config: &mut loader::MergedConfig,
    state: &State,
    installed_snapshot: &InstalledSnapshot,
    managers: &ManagerMap,
    sync_target: &SyncTarget,
) -> Result<resolver::Transaction> {
    resolve_filtered_transaction(config, state, installed_snapshot, managers, sync_target)
}

/// Check for variant package mismatches between config and installed state
pub fn check_variant_transitions(
    config: &loader::MergedConfig,
    installed_snapshot: &InstalledSnapshot,
    state: &State,
    tx: &resolver::Transaction,
    sync_target: &SyncTarget,
    options: &SyncOptions,
) -> Result<()> {
    let variant_mismatches =
        collect_variant_mismatches(config, installed_snapshot, state, tx, sync_target);
    if !variant_mismatches.is_empty() && !options.force {
        return emit_variant_transition_error(&variant_mismatches);
    }

    Ok(())
}

/// Warn about partial upgrades when system hasn't been updated recently
pub fn warn_partial_upgrade(state: &State, tx: &resolver::Transaction, options: &SyncOptions) {
    warn_partial_upgrade_impl(state, tx, options);
}

/// Display the transaction plan to the user with backend grouping
pub fn display_transaction_plan(tx: &resolver::Transaction, should_prune: bool) {
    display_transaction_plan_impl(tx, should_prune);
}

/// Display detailed dry-run simulation
/// Shows what would happen without actually executing
pub fn display_dry_run_details(
    tx: &resolver::Transaction,
    should_prune: bool,
    installed_snapshot: &InstalledSnapshot,
) {
    display_dry_run_details_impl(tx, should_prune, installed_snapshot);
}
