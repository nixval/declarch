use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "declarch",
    version = "0.2.0",
    about = "Declarative Arch packages manager",
    long_about = "A declarative package manager for Arch Linux, inspired by Nix workflow",
    next_line_help = false,
    term_width = 80
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalFlags,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser, Debug)]
pub struct GlobalFlags {
    /// Verbose output
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long = "yes", global = true)]
    pub yes: bool,

    /// Force operations
    #[arg(short = 'f', long, global = true)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize declarch configuration
    Init {
        /// Specify host name explicitly
        #[arg(long)]
        host: Option<String>,
    },

    /// Synchronize system state with configuration
    Sync {
        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,

        /// Remove packages not defined in config
        #[arg(long)]
        prune: bool,

        /// Garbage collect system orphans after sync
        #[arg(long)]
        gc: bool,

        /// Update system (paru -Syu) before syncing
        #[arg(short = 'u', long)]
        update: bool,
    },

    /// Verify configuration syntax and imports
    Check {
        #[arg(long)]
        verbose: bool,
    },

    /// Show system status and managed packages
    Info,
}
