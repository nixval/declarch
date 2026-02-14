//! Hook execution for sync workflow
//!
//! Executes pre and post-sync hooks from lifecycle configuration.

use crate::config::kdl_modules::types::LifecycleConfig;
use crate::error::Result;

/// Execute pre-sync hooks
pub fn execute_pre_sync(
    lifecycle_actions: &Option<LifecycleConfig>,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_pre_sync(lifecycle_actions, enabled, dry_run)
}

/// Execute post-sync hooks
pub fn execute_post_sync(
    lifecycle_actions: &Option<LifecycleConfig>,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_post_sync(lifecycle_actions, enabled, dry_run)
}

/// Execute success hooks
pub fn execute_on_success(
    lifecycle_actions: &Option<LifecycleConfig>,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_on_success(lifecycle_actions, enabled, dry_run)
}

/// Execute failure hooks
pub fn execute_on_failure(
    lifecycle_actions: &Option<LifecycleConfig>,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_on_failure(lifecycle_actions, enabled, dry_run)
}

/// Execute sync hooks (legacy wrapper)
#[deprecated(
    since = "0.8.0",
    note = "Use execute_pre_sync/execute_post_sync/execute_on_success/execute_on_failure explicitly"
)]
#[allow(dead_code)]
pub fn execute_sync_hooks(
    lifecycle_actions: &Option<LifecycleConfig>,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_pre_sync(lifecycle_actions, enabled, dry_run)
}
