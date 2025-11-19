use clap::Parser;
use declarch::cli::args::{Cli, Command};
use declarch::utils::output;
use declarch::commands;
use std::process::exit;

fn main() {
    ctrlc::set_handler(move || {
        println!();
        output::warning("Operation cancelled by user (Signal Received).");
        exit(130);
    }).expect("Error setting Ctrl-C handler");

    let args = Cli::parse();

    if let Err(e) = check_requirements() {
        output::error(&format!("System Requirement Error: {}", e));
        exit(1);
    }

    match run(&args) {
        Ok(_) => exit(0),
        Err(e) => {
            output::error(&format!("{}", e));
            exit(1);
        }
    }
}

fn check_requirements() -> declarch::utils::errors::Result<()> {
    if which::which("pacman").is_err() {
        return Err(declarch::utils::errors::DeclarchError::Other("pacman not found!".into()));
    }
    if which::which("git").is_err() {
        output::warning("git not found. Installing AUR packages might fail.");
    }
    Ok(())
}

fn run(args: &Cli) -> declarch::utils::errors::Result<()> {
    match &args.command {
        Some(Command::Init { host }) => {
            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                force: args.global.force,
            })
        }
        Some(Command::Sync { dry_run, prune, gc, update }) => {
            commands::sync::run(commands::sync::SyncOptions {
                dry_run: *dry_run,
                prune: *prune,
                gc: *gc,
                update: *update,
                yes: args.global.yes,
            })
        }
        Some(Command::Check { verbose }) => {
            commands::check::run(*verbose)
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
