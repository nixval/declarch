// DCL alias - wrapper for declarch command
// This file provides the 'dcl' command as a shorthand for 'declarch'

use clap::Parser;
use declarch::cli::args::{Cli, Command};
use declarch::commands;
use declarch::ui as output;
use std::process::exit;

fn main() {
    // Signal Handling
    ctrlc::set_handler(move || {
        println!();
        output::warning("Operation cancelled by user.");
        exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    // Parse & Run
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
        }) => commands::init::run(commands::init::InitOptions {
            host: host.clone(),
            path: path.clone(),
            force: args.global.force,
            skip_soar_install: *skip_soar_install,
        }),
        Some(Command::Sync {
            dry_run,
            prune,
            gc,
            update,
            target,
            noconfirm,
            hooks,
            skip_soar_install,
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
        }),
        Some(Command::Check {
            verbose,
            duplicates,
            conflicts,
            backend,
        }) => {
            let is_verbose = *verbose || args.global.verbose;
            commands::check::run(is_verbose, *duplicates, *conflicts, backend.clone())
        }
        Some(Command::Info) => commands::info::run(),
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
        }),
        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
