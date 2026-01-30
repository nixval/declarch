use clap::Parser;
use declarch::cli::args::{Cli, Command, SettingsCommand};
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
            dry_run,
            prune,
            gc,
            update,
            target,
            noconfirm,
            hooks,
            skip_soar_install,
            modules,
        }) => commands::sync::run(commands::sync::SyncOptions {
            dry_run: *dry_run,
            prune: *prune,
            gc: *gc,
            update: *update,
            yes: args.global.yes,
            force: args.global.force,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            skip_soar_install: *skip_soar_install,
            modules: modules.clone(),
        }),
        Some(Command::Check {
            duplicates,
            conflicts,
            only_duplicates,
            only_conflicts,
            backend,
            diff,
            validate,
            benchmark,
            modules,
        }) => {
            commands::check::run(
                args.global.verbose,
                *duplicates,
                *conflicts,
                *only_duplicates,
                *only_conflicts,
                backend.clone(),
                *diff,
                *validate,
                *benchmark,
                modules.clone(),
            )
        }
        Some(Command::Info { doctor, debug, backend, package }) => {
            commands::info::run(commands::info::InfoOptions {
                doctor: *doctor,
                debug: *debug,
                format: args.global.format.clone(),
                backend: backend.clone(),
                package: package.clone(),
            })
        }
        Some(Command::List {
            backend,
            orphans,
            synced,
        }) => commands::list::run(commands::list::ListOptions {
            backend: backend.clone(),
            orphans: *orphans,
            synced: *synced,
            format: args.global.format.clone(),
        }),
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
                SettingsCommand::Set { key, value } => {
                    commands::settings::SettingsCommand::Set {
                        key: key.clone(),
                        value: value.clone(),
                    }
                }
                SettingsCommand::Get { key } => {
                    commands::settings::SettingsCommand::Get {
                        key: key.clone(),
                    }
                }
                SettingsCommand::Show => commands::settings::SettingsCommand::Show,
                SettingsCommand::Reset { key } => {
                    commands::settings::SettingsCommand::Reset {
                        key: key.clone(),
                    }
                }
            };

            commands::settings::run(cmd)
        }
        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
