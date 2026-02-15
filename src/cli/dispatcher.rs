//! Command dispatcher
//!
//! Routes CLI commands to their appropriate handlers and manages
//! deprecated flag handling.

use crate::cli::args::{Cli, Command, LintMode, SyncCommand};
use crate::commands;
use crate::error::{DeclarchError, Result};
use crate::ui as output;

use super::deprecated::{
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
            restore_backends,
            restore_declarch,
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

            // Handle restore flags
            if *restore_backends {
                return commands::init::restore_backends();
            }
            if *restore_declarch {
                return commands::init::restore_declarch(host.clone());
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

        Some(Command::Sync { command, gc }) => {
            match command {
                Some(SyncCommand::Cache { backend }) => {
                    commands::cache::run(commands::cache::CacheOptions {
                        backends: if backend.is_empty() {
                            None
                        } else {
                            Some(backend.clone())
                        },
                        verbose: args.global.verbose,
                    })
                }
                Some(SyncCommand::Upgrade { backend, no_sync }) => {
                    commands::upgrade::run(commands::upgrade::UpgradeOptions {
                        backends: if backend.is_empty() {
                            None
                        } else {
                            Some(backend.clone())
                        },
                        no_sync: *no_sync,
                        verbose: args.global.verbose,
                    })
                }
                _ => {
                    // Handle other sync subcommands (Sync, Preview, Update, Prune)
                    let (has_deprecated_flags, deprecated_command, new_cmd_str) =
                        handle_deprecated_sync_flags(false, false, false, *gc);

                    // Use the command from subcommand if provided, otherwise use deprecated flags
                    let sync_cmd = command
                        .clone()
                        .unwrap_or_else(|| deprecated_command.clone());

                    // Show deprecation warning if old flags were used
                    if has_deprecated_flags {
                        show_deprecation_warning(new_cmd_str);
                    }

                    // Convert and execute
                    let options =
                        sync_command_to_options(&sync_cmd, args.global.yes, args.global.force);
                    commands::sync::run(options)
                }
            }
        }

        Some(Command::Info {
            query,
            doctor,
            plan,
            list,
            orphans,
            synced,
            backend,
            package,
            profile,
            host,
            modules,
        }) => {
            let mut mode_count = 0u8;
            if *doctor {
                mode_count += 1;
            }
            if *plan {
                mode_count += 1;
            }
            if query.is_some() {
                mode_count += 1;
            }
            if *list || *orphans || *synced {
                mode_count += 1;
            }
            if mode_count > 1 {
                return Err(DeclarchError::Other(
                    "Use only one info mode at a time: status, query, --plan, --doctor, or --list/--orphans/--synced".to_string(),
                ));
            }

            if *doctor {
                return commands::info::run(commands::info::InfoOptions {
                    doctor: true,
                    format: args.global.format.clone(),
                    backend: backend.clone(),
                    package: package.clone(),
                    verbose: args.global.verbose,
                });
            }

            if *list || *orphans || *synced {
                return commands::list::run(commands::list::ListOptions {
                    backend: backend.clone(),
                    orphans: *orphans,
                    synced: *synced,
                    format: args.global.format.clone(),
                });
            }

            if *plan || query.is_some() {
                return commands::explain::run(commands::explain::ExplainOptions {
                    query: if *plan { None } else { query.clone() },
                    target: if *plan {
                        Some("sync-plan".to_string())
                    } else {
                        None
                    },
                    profile: profile.clone(),
                    host: host.clone(),
                    modules: modules.clone(),
                });
            }

            commands::info::run(commands::info::InfoOptions {
                doctor: false,
                format: args.global.format.clone(),
                backend: backend.clone(),
                package: package.clone(),
                verbose: args.global.verbose,
            })
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

        Some(Command::Edit {
            target,
            preview,
            number,
            create,
            auto_format,
            validate_only,
            backup,
        }) => commands::edit::run(commands::edit::EditOptions {
            target: target.clone(),
            dry_run: args.global.dry_run,
            preview: *preview,
            number: *number,
            create: *create,
            auto_format: *auto_format,
            validate_only: *validate_only,
            backup: *backup,
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

        Some(Command::Search {
            query,
            backends,
            limit,
            installed_only,
            available_only,
            local,
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
                backends: if backends.is_empty() {
                    None
                } else {
                    Some(backends.clone())
                },
                limit: parsed_limit,
                installed_only: *installed_only,
                available_only: *available_only,
                local: *local,
            })
        }
        Some(Command::Lint {
            strict,
            fix,
            mode,
            backend,
            diff,
            benchmark,
            profile,
            host,
            modules,
        }) => commands::lint::run(commands::lint::LintOptions {
            strict: *strict,
            fix: *fix,
            mode: match mode {
                LintMode::All => commands::lint::LintMode::All,
                LintMode::Validate => commands::lint::LintMode::Validate,
                LintMode::Duplicates => commands::lint::LintMode::Duplicates,
                LintMode::Conflicts => commands::lint::LintMode::Conflicts,
            },
            backend: backend.clone(),
            diff: *diff,
            benchmark: *benchmark,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
        }),

        Some(Command::Completions { shell }) => commands::completions::run(*shell),

        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
