//! Deprecated flag handling
//!
//! Handles conversion from deprecated CLI flags to new command structure
//! and shows appropriate warnings to users.

use crate::cli::args::{CheckCommand, InfoCommand, ListCommand, SyncCommand};
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
    let deprecated_command = if dry_run {
        SyncCommand::Preview {
            gc,
            target: None,
            noconfirm: false,
            hooks: false,
            skip_soar_install: false,
            modules: vec![],
            diff: false,
        }
    } else if update {
        SyncCommand::Update {
            gc,
            target: None,
            noconfirm: false,
            hooks: false,
            skip_soar_install: false,
            modules: vec![],
        }
    } else if prune {
        SyncCommand::Prune {
            gc,
            target: None,
            noconfirm: false,
            hooks: false,
            skip_soar_install: false,
            modules: vec![],
        }
    } else {
        SyncCommand::Sync {
            gc,
            target: None,
            noconfirm: false,
            hooks: false,
            skip_soar_install: false,
            modules: vec![],
        }
    };

    let new_cmd = match deprecated_command {
        SyncCommand::Preview { .. } => "declarch sync preview",
        SyncCommand::Update { .. } => "declarch sync update",
        SyncCommand::Prune { .. } => "declarch sync prune",
        SyncCommand::Sync { .. } => "declarch sync",
    };

    (true, deprecated_command, new_cmd)
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
            },
        )
    } else if conflicts {
        (
            true,
            CheckCommand::Conflicts {
                backend: None,
                diff: false,
            },
        )
    } else if validate {
        (
            true,
            CheckCommand::Validate {
                benchmark: false,
                modules: vec![],
            },
        )
    } else {
        (
            false,
            CheckCommand::All {
                backend: None,
                diff: false,
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
                summary: false,
            },
        )
    }
}

/// Convert deprecated list flags to ListCommand
pub fn handle_deprecated_list_flags(
    orphans: bool,
    synced: bool,
) -> (bool, ListCommand, &'static str) {
    let (has_deprecated, deprecated_command) = if orphans {
        (true, ListCommand::Orphans { backend: None })
    } else if synced {
        (true, ListCommand::Synced { backend: None })
    } else {
        (false, ListCommand::All { backend: None })
    };

    let new_cmd = match deprecated_command {
        ListCommand::Orphans { .. } => "declarch list orphans",
        ListCommand::Synced { .. } => "declarch list synced",
        ListCommand::All { .. } => "declarch list",
    };

    (has_deprecated, deprecated_command, new_cmd)
}

/// Show deprecation warning for deprecated flag usage
pub fn show_deprecation_warning(new_command: &str) {
    output::warning(&format!(
        "Deprecated flag usage. Please use: {}",
        new_command.cyan()
    ));
    output::info("Old flags will be removed in a future version");
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
            noconfirm,
            hooks,
            skip_soar_install,
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
            skip_soar_install: *skip_soar_install,
            modules: modules.clone(),
            diff: false,
        },
        SyncCommand::Preview {
            gc,
            target,
            noconfirm,
            hooks,
            skip_soar_install,
            modules,
            diff,
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
            skip_soar_install: *skip_soar_install,
            modules: modules.clone(),
            diff: *diff,
        },
        SyncCommand::Update {
            gc,
            target,
            noconfirm,
            hooks,
            skip_soar_install,
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
            skip_soar_install: *skip_soar_install,
            modules: modules.clone(),
            diff: false,
        },
        SyncCommand::Prune {
            gc,
            target,
            noconfirm,
            hooks,
            skip_soar_install,
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
            skip_soar_install: *skip_soar_install,
            modules: modules.clone(),
            diff: false,
        },
    }
}
