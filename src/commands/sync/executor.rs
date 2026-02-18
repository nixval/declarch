//! Transaction execution for sync operations
//!
//! Installs, adopts, and prunes packages based on transaction plan.

mod install_ops;
mod prune;
mod retry;
mod snapshot;

use super::{InstalledSnapshot, ManagerMap, SyncOptions};
use crate::config::loader;
use crate::constants::{BACKEND_OPERATION_MAX_RETRIES, BACKEND_RETRY_DELAY_MS};
use crate::core::{resolver, types::PackageId};
use crate::error::Result;
use install_ops::execute_installations;
use prune::execute_pruning;
use snapshot::build_installed_snapshot;

/// Maximum retry attempts for failed backend operations
const MAX_RETRIES: u32 = BACKEND_OPERATION_MAX_RETRIES;
/// Delay between retries (in milliseconds)
const RETRY_DELAY_MS: u64 = BACKEND_RETRY_DELAY_MS;

/// Execute transaction (install, adopt, prune)
pub fn execute_transaction(
    transaction: &resolver::Transaction,
    managers: &ManagerMap,
    config: &loader::MergedConfig,
    options: &SyncOptions,
    hooks_enabled: bool,
) -> Result<Vec<PackageId>> {
    let mut installed_snapshot = build_installed_snapshot(managers)?;

    // Execute installations
    let successfully_installed = execute_installations(
        transaction,
        managers,
        config,
        options,
        hooks_enabled,
        &mut installed_snapshot,
    )?;

    // Execute pruning if enabled
    if options.prune && !transaction.to_prune.is_empty() {
        execute_pruning(
            config,
            transaction,
            managers,
            options,
            hooks_enabled,
            &installed_snapshot,
        )?;
    }

    Ok(successfully_installed)
}
