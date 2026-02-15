//! Deprecated flag handling
//!
//! Handles conversion from deprecated CLI flags to new command structure
//! and shows appropriate warnings to users.

use crate::cli::args::{CheckCommand, InfoCommand, ListSubcommand, SyncCommand};
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
            modules: vec![],
        }
    } else if update {
        SyncCommand::Update {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            modules: vec![],
        }
    } else if prune {
        SyncCommand::Prune {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            modules: vec![],
        }
    } else {
        SyncCommand::Sync {
            gc,
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
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

/// Convert deprecated check flags to CheckCommand
pub fn handle_deprecated_check_flags(
    duplicates: bool,
    conflicts: bool,
    validate: bool,
) -> (bool, CheckCommand, &'static str) {
    let (has_deprecated, deprecated_command) = if duplicates {
        (
            true,
            CheckCommand::Duplicates {
                backend: None,
                diff: false,
                fix: false,
            },
        )
    } else if conflicts {
        (
            true,
            CheckCommand::Conflicts {
                backend: None,
                diff: false,
                fix: false,
            },
        )
    } else if validate {
        (
            true,
            CheckCommand::Validate {
                benchmark: false,
                fix: false,
                modules: vec![],
            },
        )
    } else {
        (
            false,
            CheckCommand::All {
                backend: None,
                diff: false,
                fix: false,
                benchmark: false,
                modules: vec![],
            },
        )
    };

    let new_cmd = match deprecated_command {
        CheckCommand::Duplicates { .. } => "declarch check duplicates",
        CheckCommand::Conflicts { .. } => "declarch check conflicts",
        CheckCommand::Validate { .. } => "declarch check validate",
        CheckCommand::All { .. } => "declarch check",
    };

    (has_deprecated, deprecated_command, new_cmd)
}

/// Convert deprecated info flags to InfoCommand
pub fn handle_deprecated_info_flags(doctor: bool) -> (bool, InfoCommand) {
    if doctor {
        (
            true,
            InfoCommand::Doctor {
                debug: false,
                backend: None,
                package: None,
            },
        )
    } else {
        (
            false,
            InfoCommand::Status {
                debug: false,
                backend: None,
                package: None,
            },
        )
    }
}

/// Convert deprecated list flags to ListSubcommand
pub fn handle_deprecated_list_flags(orphans: bool, synced: bool) -> (bool, ListSubcommand, String) {
    let (has_deprecated, deprecated_command) = if orphans {
        (true, ListSubcommand::Orphans { backend: None })
    } else if synced {
        (true, ListSubcommand::Synced { backend: None })
    } else {
        (false, ListSubcommand::All { backend: None })
    };

    let new_cmd = match deprecated_command {
        ListSubcommand::Orphans { .. } => "declarch info list orphans".to_string(),
        ListSubcommand::Synced { .. } => "declarch info list synced".to_string(),
        ListSubcommand::All { .. } => "declarch info list".to_string(),
    };

    (has_deprecated, deprecated_command, new_cmd)
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
            modules: modules.clone(),
            diff: *diff,
        },
        SyncCommand::Preview {
            gc,
            target,
            noconfirm,
            hooks,
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
            modules: modules.clone(),
            diff: false,
        },
        SyncCommand::Update {
            gc,
            target,
            diff,
            noconfirm,
            hooks,
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
            modules: modules.clone(),
            diff: *diff,
        },
        SyncCommand::Prune {
            gc,
            target,
            diff,
            noconfirm,
            hooks,
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
            modules: modules.clone(),
            diff: *diff,
        },
        // Cache and Upgrade are handled directly in dispatcher, not through sync options
        SyncCommand::Cache { .. } | SyncCommand::Upgrade { .. } => {
            unreachable!("Cache and Upgrade should be handled directly in dispatcher")
        }
    }
}
