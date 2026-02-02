//! Declarch - Declarative Package Manager for Linux
//!
//! A declarative package manager that allows you to define your desired
//! software packages in a simple configuration file and keep your system
//! in sync with that configuration.

use clap::Parser;
use declarch::cli::args::Cli;
use declarch::error_suggestions::ErrorSuggestions;
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

    if let Err(e) = declarch::cli::dispatcher::dispatch(&args) {
        // Display error with helpful suggestion
        eprintln!("{}", e.display_with_suggestion());
        exit(1);
    }
}
