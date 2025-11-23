use clap::Parser;
use declarch::cli::args::{Cli, Command};
use declarch::ui as output; 
use declarch::commands;
use std::process::exit;

fn main() {
    ctrlc::set_handler(move || {
        println!();
        output::warning("Operation cancelled by user.");
        exit(130);
    }).expect("Error setting Ctrl-C handler");

    let args = Cli::parse();


    if let Err(e) = run(&args) {
        output::error(&format!("{}", e));
        exit(1);
    }
}

fn run(args: &Cli) -> declarch::error::Result<()> {
    match &args.command {
        Some(Command::Init { host }) => {
            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                force: args.global.force,
            })
        }
        Some(Command::Sync { dry_run, prune, gc, update, target, noconfirm }) => {
            commands::sync::run(commands::sync::SyncOptions {
                dry_run: *dry_run,
                prune: *prune,
                gc: *gc,
                update: *update,
                yes: args.global.yes,
                target: target.clone(),
                noconfirm: *noconfirm,

            })
        }
        Some(Command::Check { verbose, duplicates }) => {

            let is_verbose = *verbose || args.global.verbose;
            commands::check::run(is_verbose, *duplicates)
        }
        Some(Command::Info) => {
            commands::info::run()
        }
        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
    }
}
