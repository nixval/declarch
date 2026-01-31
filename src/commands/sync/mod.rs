//! Sync command orchestration
//!
//! This module orchestrates the sync workflow by coordinating:
//! - Transaction planning (planner.rs)
//! - Package installation/removal (executor.rs)
//! - State updates (state_sync.rs)
//! - Hook execution (hooks.rs)
//! - Variant matching (variants.rs)

mod planner;
mod executor;
mod state_sync;
mod hooks;
mod variants;

// Re-export public API
pub use planner::Transaction;
pub use planner::create_transaction;
pub use executor::execute_transaction;
pub use state_sync::update_state;
pub use hooks::execute_sync_hooks;

use crate::config::loader;
use crate::constants::CRITICAL_PACKAGES;
use crate::core::{
    resolver,
    types::{PackageId, PackageMetadata, SyncTarget},
};
use crate::error::{DeclarchError, Result};
use crate::packages::{PackageManager, create_manager};
use crate::state::{
    self,
    types::{Backend, PackageState, State},
};
use crate::ui as output;
use crate::utils::distro::DistroType;
use crate::utils::install;
use crate::utils::paths;
use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

// Type aliases to reduce complexity
pub type InstalledSnapshot = HashMap<PackageId, PackageMetadata>;
pub type ManagerMap = HashMap<Backend, Box<dyn PackageManager>>;

/// AUR package variant suffixes for smart matching
const AUR_SUFFIXES: &[&str] = &["-bin", "-git", "-hg", "-nightly", "-beta", "-wayland"];

#[derive(Debug)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
    pub gc: bool,
    pub update: bool,
    pub yes: bool,
    pub force: bool,
    pub target: Option<String>,
    pub noconfirm: bool,
    pub hooks: bool,
    pub skip_soar_install: bool,
    pub modules: Vec<String>,
}

pub fn run(options: SyncOptions) -> Result<()> {
    // 1. Target Resolution
    let sync_target = resolve_target(&options.target);

    // 2. Load Config
    let config_path = paths::config_file()?;
    let mut config = if !options.modules.is_empty() {
        // Check if this is selective sync (single module, no target specified)
        // or loading additional modules
        if options.modules.len() == 1 && options.target.is_none() {
            // Selective sync: load ONLY this module (no verbose message)
            load_single_module(&config_path, &options.modules[0])?
        } else {
            // Load additional modules
            load_config_with_modules(&config_path, &options.modules)?
        }
    } else {
        loader::load_root_config(&config_path)?
    };

    // Execute pre-sync hooks
    execute_sync_hooks(&config.lifecycle_actions, options.hooks, options.dry_run)?;

    // 3. System Update
    perform_system_update(&options)?;

    // 4. Initialize Managers & Snapshot
    let (mut installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 5. Load State & Resolve
    let mut state = state::io::load_state()?;

    // ==========================================
    // PHASE 2: VARIANT DETECTION
    // ==========================================
    let variant_updates = variants::detect_and_resolve_variant_transitions(
        &mut state,
        &installed_snapshot,
        &sync_target,
    )?;

    // ==========================================
    // PHASE 3: TRANSACTION PLANNING
    // ==========================================
    let tx = create_transaction(
        &config,
        &state,
        &installed_snapshot,
        &sync_target,
        &options,
    )?;

    // ==========================================
    // PHASE 4: EXECUTE TRANSACTION
    // ==========================================
    if tx.to_install.is_empty()
        && tx.to_adopt.is_empty()
        && tx.to_update_project_metadata.is_empty()
        && (!options.prune || tx.to_prune.is_empty())
    {
        output::success("System is in sync.");
        if options.update {
            state.meta.last_update = Some(Utc::now());
            state::io::save_state_locked(&state)?;
        }
        // Execute post-sync hooks even when system is in sync
        execute_sync_hooks(
            &config.lifecycle_actions,
            options.hooks,
            options.dry_run,
        )?;
        return Ok(());
    }

    if options.dry_run {
        output::info("Dry run mode - no changes will be made");
        return Ok(());
    }

    execute_transaction(&tx, &managers, &config, &options)?;

    // ==========================================
    // PHASE 5: UPDATE STATE
    // ==========================================
    let state = update_state(&state, &tx, &installed_snapshot, &options)?;

    // ==========================================
    // PHASE 6: POST-SYNC HOOKS
    // ==========================================
    execute_sync_hooks(
        &config.lifecycle_actions,
        options.hooks,
        options.dry_run,
    )?;

    Ok(())
}
