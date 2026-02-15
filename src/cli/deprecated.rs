//! Deprecated flag handling
//!
//! Handles conversion from deprecated CLI flags to new command structure
//! and shows appropriate warnings to users.

use crate::cli::args::SyncCommand;
use crate::ui as output;
use colored::Colorize;

/// Convert deprecated sync flags to SyncCommand with deprecation warning
///
/// Returns (has_deprecated_flags, sync_command, new_command_string)
pub fn handle_deprecated_sync_flags(
    dry_run: bool,
    update: bool,
    prune: bool,
    gc: bool,
) -> (bool, SyncCommand, &'static str) {
    // Check if any deprecated flag is actually used
    let has_deprecated_flags = dry_run || update || prune;

    let deprecated_command = if dry_run {
        SyncCommand::Preview {
            gc,
            target: None,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: vec![],
        }
    } else if update {
        SyncCommand::Update {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: vec![],
        }
    } else if prune {
        SyncCommand::Prune {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: vec![],
        }
    } else {
        SyncCommand::Sync {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: vec![],
        }
    };

    let new_cmd = match deprecated_command {
        SyncCommand::Preview { .. } => "declarch sync preview",
        SyncCommand::Update { .. } => "declarch sync update",
        SyncCommand::Prune { .. } => "declarch sync prune",
        SyncCommand::Sync { .. } => "declarch sync",
        SyncCommand::Cache { .. } => "declarch sync cache",
        SyncCommand::Upgrade { .. } => "declarch sync upgrade",
    };

    (has_deprecated_flags, deprecated_command, new_cmd)
}

/// Show deprecation warning for deprecated flag usage
pub fn show_deprecation_warning(new_command: &str) {
    output::warning(&format!(
        "Deprecated flag usage. Please use: {}",
        new_command.cyan()
    ));
    output::info("Old flags will be removed in a future major/minor release");
}

/// Convert SyncCommand to sync::run options
pub fn sync_command_to_options(
    sync_cmd: &SyncCommand,
    yes: bool,
    force: bool,
) -> crate::commands::sync::SyncOptions {
    use crate::commands::sync;

    match sync_cmd {
        SyncCommand::Sync {
            gc,
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        } => sync::SyncOptions {
            dry_run: false,
            prune: false,
            update: false,
            gc: *gc,
            yes,
            force,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: *diff,
            format: None,
            output_version: None,
        },
        SyncCommand::Preview {
            gc,
            target,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        } => sync::SyncOptions {
            dry_run: true,
            prune: false,
            update: false,
            gc: *gc,
            yes: false,
            force: false,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: false,
            format: None,
            output_version: None,
        },
        SyncCommand::Update {
            gc,
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        } => sync::SyncOptions {
            dry_run: false,
            prune: false,
            update: true,
            gc: *gc,
            yes: false,
            force: false,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: *diff,
            format: None,
            output_version: None,
        },
        SyncCommand::Prune {
            gc,
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        } => sync::SyncOptions {
            dry_run: false,
            prune: true,
            update: false,
            gc: *gc,
            yes: false,
            force: false,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: *diff,
            format: None,
            output_version: None,
        },
        // Cache and Upgrade are handled directly in dispatcher, not through sync options
        SyncCommand::Cache { .. } | SyncCommand::Upgrade { .. } => {
            unreachable!("Cache and Upgrade should be handled directly in dispatcher")
        }
    }
}
