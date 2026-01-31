//! Hook execution for sync workflow
//!
//! Executes pre and post-sync hooks from lifecycle configuration.

use crate::config::types::LifecycleActions;
use crate::error::Result;

/// Execute pre-sync hooks
pub fn execute_pre_sync(
    lifecycle_actions: &LifecycleActions,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_pre_sync(lifecycle_actions, enabled, dry_run)
}

/// Execute post-sync hooks
pub fn execute_post_sync(
    lifecycle_actions: &LifecycleActions,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    crate::commands::hooks::execute_post_sync(lifecycle_actions, enabled, dry_run)
}

/// Execute sync hooks (both pre and post)
pub fn execute_sync_hooks(
    lifecycle_actions: &LifecycleActions,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    execute_post_sync(lifecycle_actions, enabled, dry_run)
}
