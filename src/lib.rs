pub mod backends;
pub mod cli;
pub mod commands;
pub mod config;
pub mod constants;
pub mod core;
pub mod error;
pub mod packages;
pub mod project_identity;
pub mod state;
pub mod traits;
pub mod ui;
pub mod utils;

use clap::Parser;
use std::process::exit;

/// Run declarch CLI entrypoint.
pub fn run_cli() {
    // 0. Initialize color settings (must be first)
    ui::init_colors();

    // 1. Signal Handling (mark cancellation and let command flow rollback safely)
    ctrlc::set_handler(move || {
        eprintln!();
        ui::mark_interrupted();
        ui::warning("Operation cancelled by user.");
    })
    .expect("Error setting Ctrl-C handler");

    // 2. Parse & Run
    let args = cli::args::Cli::parse();
    ui::set_quiet(args.global.quiet);
    ui::set_verbose(args.global.verbose);

    if let Err(e) = cli::dispatcher::dispatch(&args) {
        ui::error(&format!("{}", e));
        exit(1);
    }
}
