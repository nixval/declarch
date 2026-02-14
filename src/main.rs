//! Declarch - Declarative Package Manager for Linux
//!
//! A declarative package manager that allows you to define your desired
//! software packages in a simple configuration file and keep your system
//! in sync with that configuration.

use clap::Parser;
use declarch::cli::args::Cli;
use declarch::ui as output;
use std::process::exit;

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
    output::set_quiet(args.global.quiet);

    if let Err(e) = declarch::cli::dispatcher::dispatch(&args) {
        output::error(&format!("{}", e));
        exit(1);
    }
}
