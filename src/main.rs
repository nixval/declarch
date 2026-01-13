use clap::Parser;
use declarch::cli::args::{Cli, Command};
use declarch::ui as output; 
use declarch::commands;
use std::process::exit;

// Removed unused 'Command as SysCommand' to fix warning
// If you implement the root check later, add it back.

fn main() {
    // 1. Signal Handling
    ctrlc::set_handler(move || {
        println!();
        output::warning("Operation cancelled by user.");
        exit(130);
    }).expect("Error setting Ctrl-C handler");

    // 2. Parse & Run
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        output::error(&format!("{}", e));
        exit(1);
    }
}

fn run(args: &Cli) -> declarch::error::Result<()> {
    match &args.command {
        Some(Command::Init { host, path, skip_soar_install }) => {
            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                // Now 'path' is recognized because we updated args.rs
                path: path.clone(),
                force: args.global.force,
                skip_soar_install: *skip_soar_install,
            })
        }
        Some(Command::Sync { dry_run, prune, gc, update, target, noconfirm, skip_soar_install }) => {
            commands::sync::run(commands::sync::SyncOptions {
                dry_run: *dry_run,
                prune: *prune,
                gc: *gc,
                update: *update,
                yes: args.global.yes,
                force: args.global.force,
                target: target.clone(),
                noconfirm: *noconfirm,
                skip_soar_install: *skip_soar_install,
            })
        }
        Some(Command::Check { verbose, duplicates }) => {
            let is_verbose = *verbose || args.global.verbose;
            commands::check::run(is_verbose, *duplicates)
        }
        Some(Command::Info) => {
            commands::info::run()
        }
        Some(Command::Switch { old_package, new_package, backend, dry_run }) => {
            commands::switch::run(commands::switch::SwitchOptions {
                old_package: old_package.clone(),
                new_package: new_package.clone(),
                backend: backend.clone(),
                dry_run: *dry_run,
                yes: args.global.yes,
                force: args.global.force,
            })
        }
        Some(Command::Edit { target }) => {
            commands::edit::run(commands::edit::EditOptions {
                target: target.clone(),
            })
        }
        Some(Command::Completions { shell }) => {
            commands::completions::run(*shell)
        }
        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
