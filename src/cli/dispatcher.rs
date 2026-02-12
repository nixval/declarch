//! Command dispatcher
//!
//! Routes CLI commands to their appropriate handlers and manages
//! deprecated flag handling.

use crate::cli::args::{CheckCommand, Cli, Command, InfoCommand, ListCommand, SettingsCommand};
use crate::commands;
use crate::error::{DeclarchError, Result};
use crate::ui as output;

use super::deprecated::{
    handle_deprecated_check_flags, handle_deprecated_info_flags, handle_deprecated_list_flags,
    handle_deprecated_sync_flags, show_deprecation_warning, sync_command_to_options,
};

/// Dispatch the parsed CLI command to the appropriate handler
pub fn dispatch(args: &Cli) -> Result<()> {
    match &args.command {
        Some(Command::Init {
            host,
            path,
            backend,
            list,
            local,
        }) => {
            // Handle --list flag first
            if let Some(what) = list {
                if what == "backends" {
                    return commands::init::list_available_backends();
                } else if what == "modules" {
                    return commands::init::list_available_modules();
                } else {
                    return Err(DeclarchError::Other(format!(
                        "Unknown list target: '{}'. Available: backends, modules",
                        what
                    )));
                }
            }
            
            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                path: path.clone(),
                backends: backend.clone(),
                force: args.global.force,
                yes: args.global.yes,
                local: *local,
            })
        }

        Some(Command::Sync {
            command,
            dry_run,
            prune,
            gc,
            update,
        }) => {
            // Handle deprecated flags
            let (has_deprecated_flags, deprecated_command, new_cmd_str) =
                handle_deprecated_sync_flags(*dry_run, *update, *prune, *gc);

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let sync_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                show_deprecation_warning(new_cmd_str);
            }

            // Convert and execute
            let options = sync_command_to_options(&sync_cmd, args.global.yes, args.global.force);
            commands::sync::run(options)
        }

        Some(Command::Check {
            command,
            duplicates,
            conflicts,
            only_duplicates: _,
            only_conflicts: _,
            validate,
        }) => {
            // Handle deprecated flags
            let (has_deprecated_flags, deprecated_command, new_cmd_str) =
                handle_deprecated_check_flags(*duplicates, *conflicts, *validate);

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let check_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                show_deprecation_warning(new_cmd_str);
            }

            // Map CheckCommand to check::run parameters
            match check_cmd {
                CheckCommand::All {
                    backend,
                    diff,
                    benchmark,
                    modules,
                } => commands::check::run(
                    args.global.verbose,
                    true,  // check_duplicates
                    true,  // check_conflicts
                    false, // only_duplicates
                    false, // only_conflicts
                    backend,
                    diff,
                    false, // validate_only
                    benchmark,
                    modules,
                ),
                CheckCommand::Duplicates { backend, diff } => {
                    commands::check::run(
                        args.global.verbose,
                        true,  // check_duplicates
                        false, // check_conflicts
                        true,  // only_duplicates
                        false, // only_conflicts
                        backend,
                        diff,
                        false, // validate_only
                        false, // benchmark
                        vec![],
                    )
                }
                CheckCommand::Conflicts { backend, diff } => {
                    commands::check::run(
                        args.global.verbose,
                        false, // check_duplicates
                        true,  // check_conflicts
                        false, // only_duplicates
                        true,  // only_conflicts
                        backend,
                        diff,
                        false, // validate_only
                        false, // benchmark
                        vec![],
                    )
                }
                CheckCommand::Validate { benchmark, modules } => {
                    commands::check::run(
                        args.global.verbose,
                        false, // check_duplicates
                        false, // check_conflicts
                        false, // only_duplicates
                        false, // only_conflicts
                        None,  // backend
                        false, // diff
                        true,  // validate_only
                        benchmark,
                        modules,
                    )
                }
            }
        }

        Some(Command::Info { command, doctor }) => {
            // Handle deprecated flag
            let (has_deprecated_flag, deprecated_command) = handle_deprecated_info_flags(*doctor);

            // Use the command from subcommand if provided, otherwise use deprecated flag
            let info_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flag was used
            if has_deprecated_flag {
                show_deprecation_warning("declarch info doctor");
            }

            // Map InfoCommand to info::run parameters
            match info_cmd {
                InfoCommand::Status {
                    debug,
                    backend,
                    package,
                } => commands::info::run(commands::info::InfoOptions {
                    doctor: false,
                    debug,
                    format: args.global.format.clone(),
                    backend,
                    package,
                }),
                InfoCommand::Doctor {
                    debug,
                    backend,
                    package,
                } => commands::info::run(commands::info::InfoOptions {
                    doctor: true,
                    debug,
                    format: args.global.format.clone(),
                    backend,
                    package,
                }),
            }
        }

        Some(Command::List {
            command,
            orphans,
            synced,
        }) => {
            // Handle deprecated flags
            let (has_deprecated_flags, deprecated_command, new_cmd_str) =
                handle_deprecated_list_flags(*orphans, *synced);

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let list_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                show_deprecation_warning(new_cmd_str);
            }

            // Map ListCommand to list::run parameters
            match list_cmd {
                ListCommand::All { backend } => commands::list::run(commands::list::ListOptions {
                    backend,
                    orphans: false,
                    synced: false,
                    format: args.global.format.clone(),
                }),
                ListCommand::Orphans { backend } => {
                    commands::list::run(commands::list::ListOptions {
                        backend,
                        orphans: true,
                        synced: false,
                        format: args.global.format.clone(),
                    })
                }
                ListCommand::Synced { backend } => {
                    commands::list::run(commands::list::ListOptions {
                        backend,
                        orphans: false,
                        synced: true,
                        format: args.global.format.clone(),
                    })
                }
            }
        }

        Some(Command::Switch {
            old_package,
            new_package,
            backend,
            dry_run,
        }) => commands::switch::run(commands::switch::SwitchOptions {
            old_package: old_package.clone(),
            new_package: new_package.clone(),
            backend: backend.clone(),
            dry_run: *dry_run,
            yes: args.global.yes,
            force: args.global.force,
        }),

        Some(Command::Edit { target }) => commands::edit::run(commands::edit::EditOptions {
            target: target.clone(),
            dry_run: args.global.dry_run,
        }),

        Some(Command::Install {
            packages,
            backend,
            module,
            no_sync,
        }) => commands::install::run(commands::install::InstallOptions {
            packages: packages.clone(),
            backend: backend.clone(),
            module: module.clone(),
            no_sync: *no_sync,
            yes: args.global.yes,
            dry_run: args.global.dry_run,
        }),

        Some(Command::Settings { command }) => {
            // Convert CLI SettingsCommand to command SettingsCommand
            let cmd = match command {
                SettingsCommand::Set { key, value } => commands::settings::SettingsCommand::Set {
                    key: key.clone(),
                    value: value.clone(),
                },
                SettingsCommand::Get { key } => {
                    commands::settings::SettingsCommand::Get { key: key.clone() }
                }
                SettingsCommand::Show => commands::settings::SettingsCommand::Show,
                SettingsCommand::Reset { key } => {
                    commands::settings::SettingsCommand::Reset { key: key.clone() }
                }
            };

            commands::settings::run(cmd)
        }

        Some(Command::Search {
            query,
            backends,
            limit,
            installed_only,
            available_only,
        }) => {
            // Parse limit option: "all" or "0" means unlimited, otherwise parse as number
            let parsed_limit = if let Some(limit_str) = limit {
                if limit_str == "all" || limit_str == "0" {
                    None // Unlimited
                } else {
                    Some(limit_str.parse::<usize>().unwrap_or(10))
                }
            } else {
                Some(10) // Default 10
            };

            commands::search::run(commands::search::SearchOptions {
                query: query.clone(),
                backends: if backends.is_empty() { None } else { Some(backends.clone()) },
                limit: parsed_limit,
                installed_only: *installed_only,
                available_only: *available_only,
            })
        }

        Some(Command::Cache { backend }) => {
            commands::cache::run(commands::cache::CacheOptions {
                backends: if backend.is_empty() { None } else { Some(backend.clone()) },
            })
        }

        Some(Command::Completions { shell }) => commands::completions::run(*shell),

        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
