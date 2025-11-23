use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "declarch",
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
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    #[arg(short = 'y', long = "yes", global = true)]
    pub yes: bool,

    #[arg(short = 'f', long, global = true)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init {
        host: Option<String>,
    },

    Sync {
        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        prune: bool,

        #[arg(long)]
        gc: bool,

        #[arg(short = 'u', long)]
        update: bool,

        #[arg(long, value_name = "TARGET")]
        target: Option<String>,


        #[arg(long)]
        noconfirm: bool,
        

        #[arg(long)]
        only_aur: bool,
        
        #[arg(long)]
        only_flatpak: bool,
    },

    Check {
        #[arg(long)]
        verbose: bool,
        
        #[arg(long)]
        duplicates: bool,
    },

    Info,
}
