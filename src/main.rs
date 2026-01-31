use clap::Parser;
use colored::Colorize;
use declarch::cli::args::{
    CheckCommand, Cli, Command, InfoCommand, ListCommand, SettingsCommand, SyncCommand,
};
use declarch::commands;
use declarch::ui as output;
use std::process::exit;

// Removed unused 'Command as SysCommand' to fix warning
// If you implement the root check later, add it back.

fn main() {
    // 0. Initialize color settings (must be first)
    output::init_colors();

    // 1. Signal Handling
    ctrlc::set_handler(move || {
        println!();
        output::warning("Operation cancelled by user.");
        exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    // 2. Parse & Run
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        output::error(&format!("{}", e));
        exit(1);
    }
}

fn run(args: &Cli) -> declarch::error::Result<()> {
    match &args.command {
        Some(Command::Init {
            host,
            path,
            skip_soar_install,
        }) => {
            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                // Now 'path' is recognized because we updated args.rs
                path: path.clone(),
                force: args.global.force,
                skip_soar_install: *skip_soar_install,
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
            let (has_deprecated_flags, deprecated_command) = if *dry_run {
                (
                    true,
                    SyncCommand::Preview {
                        gc: *gc,
                        target: None,
                        noconfirm: false,
                        hooks: false,
                        skip_soar_install: false,
                        modules: vec![],
                    },
                )
            } else if *update {
                (
                    true,
                    SyncCommand::Update {
                        gc: *gc,
                        target: None,
                        noconfirm: false,
                        hooks: false,
                        skip_soar_install: false,
                        modules: vec![],
                    },
                )
            } else if *prune {
                (
                    true,
                    SyncCommand::Prune {
                        gc: *gc,
                        target: None,
                        noconfirm: false,
                        hooks: false,
                        skip_soar_install: false,
                        modules: vec![],
                    },
                )
            } else {
                (
                    false,
                    SyncCommand::Sync {
                        gc: *gc,
                        target: None,
                        noconfirm: false,
                        hooks: false,
                        skip_soar_install: false,
                        modules: vec![],
                    },
                )
            };

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let sync_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                let new_cmd = match deprecated_command {
                    SyncCommand::Preview { .. } => "declarch sync preview",
                    SyncCommand::Update { .. } => "declarch sync update",
                    SyncCommand::Prune { .. } => "declarch sync prune",
                    SyncCommand::Sync { .. } => "declarch sync",
                };
                output::warning(&format!(
                    "Deprecated flag usage. Please use: {}",
                    new_cmd.cyan()
                ));
                output::info("Old flags will be removed in v0.7.0");
            }

            // Map SyncCommand to sync::run parameters
            match sync_cmd {
                SyncCommand::Sync {
                    gc,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                } => commands::sync::run(commands::sync::SyncOptions {
                    dry_run: false,
                    prune: false,
                    gc,
                    update: false,
                    yes: args.global.yes,
                    force: args.global.force,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                }),
                SyncCommand::Preview {
                    gc,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                } => commands::sync::run(commands::sync::SyncOptions {
                    dry_run: true,
                    prune: false,
                    gc,
                    update: false,
                    yes: args.global.yes,
                    force: args.global.force,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                }),
                SyncCommand::Update {
                    gc,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                } => commands::sync::run(commands::sync::SyncOptions {
                    dry_run: false,
                    prune: false,
                    gc,
                    update: true,
                    yes: args.global.yes,
                    force: args.global.force,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                }),
                SyncCommand::Prune {
                    gc,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                } => commands::sync::run(commands::sync::SyncOptions {
                    dry_run: false,
                    prune: true,
                    gc,
                    update: false,
                    yes: args.global.yes,
                    force: args.global.force,
                    target,
                    noconfirm,
                    hooks,
                    skip_soar_install,
                    modules,
                }),
            }
        }
        Some(Command::Check {
            command,
            duplicates,
            conflicts,
            only_duplicates,
            only_conflicts,
            validate,
        }) => {
            // Handle deprecated flags
            let (has_deprecated_flags, deprecated_command) = if *duplicates {
                (
                    true,
                    CheckCommand::Duplicates {
                        backend: None,
                        diff: false,
                    },
                )
            } else if *conflicts {
                (
                    true,
                    CheckCommand::Conflicts {
                        backend: None,
                        diff: false,
                    },
                )
            } else if *only_duplicates {
                (
                    true,
                    CheckCommand::Duplicates {
                        backend: None,
                        diff: false,
                    },
                )
            } else if *only_conflicts {
                (
                    true,
                    CheckCommand::Conflicts {
                        backend: None,
                        diff: false,
                    },
                )
            } else if *validate {
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

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let check_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                let new_cmd = match deprecated_command {
                    CheckCommand::Duplicates { .. } => "declarch check duplicates",
                    CheckCommand::Conflicts { .. } => "declarch check conflicts",
                    CheckCommand::Validate { .. } => "declarch check validate",
                    CheckCommand::All { .. } => "declarch check",
                };
                output::warning(&format!(
                    "Deprecated flag usage. Please use: {}",
                    new_cmd.cyan()
                ));
                output::info("Old flags will be removed in v0.7.0");
            }

            // Map CheckCommand to check::run parameters
            match check_cmd {
                CheckCommand::All {
                    backend,
                    diff,
                    benchmark,
                    modules,
                } => {
                    commands::check::run(
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
                    )
                }
                CheckCommand::Duplicates { backend, diff } => {
                    commands::check::run(
                        args.global.verbose,
                        true,  // check_duplicates
                        false, // check_conflicts
                        true,  // only_duplicates (skip conflicts)
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
                        true,  // only_conflicts (skip duplicates)
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
            let (has_deprecated_flag, deprecated_command) = if *doctor {
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
            };

            // Use the command from subcommand if provided, otherwise use deprecated flag
            let info_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flag was used
            if has_deprecated_flag {
                output::warning(&format!(
                    "Deprecated flag usage. Please use: {}",
                    "declarch info doctor".cyan()
                ));
                output::info("Old flags will be removed in v0.7.0");
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
            let (has_deprecated_flags, deprecated_command) = if *orphans {
                (true, ListCommand::Orphans { backend: None })
            } else if *synced {
                (true, ListCommand::Synced { backend: None })
            } else {
                (false, ListCommand::All { backend: None })
            };

            // Use the command from subcommand if provided, otherwise use deprecated flags
            let list_cmd = command
                .clone()
                .unwrap_or_else(|| deprecated_command.clone());

            // Show deprecation warning if old flags were used
            if has_deprecated_flags {
                let new_cmd = match deprecated_command {
                    ListCommand::Orphans { .. } => "declarch list orphans",
                    ListCommand::Synced { .. } => "declarch list synced",
                    ListCommand::All { .. } => "declarch list",
                };
                output::warning(&format!(
                    "Deprecated flag usage. Please use: {}",
                    new_cmd.cyan()
                ));
                output::info("Old flags will be removed in v0.7.0");
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
                backends: backends.as_ref().map(|b| b.split(',').map(|s| s.to_string()).collect()),
                limit: parsed_limit,
                installed_only: *installed_only,
                available_only: *available_only,
            })
        }
        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
