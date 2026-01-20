use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(
    name = "declarch",
    about = "Declarative package manager for Linux",
    long_about = "A declarative package manager for Linux, inspired by Nix workflow. Supports AUR, Flatpak, and Soar.",
    version,
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
    ///
    /// With no arguments: Creates root config (~/.config/declarch/declarch.kdl)
    ///
    /// With SOURCE: Fetches config from remote repository
    ///   - user/repo              Fetch from GitHub (user/repo/main/declarch.kdl)
    ///   - user/repo:variant      Fetch specific config variant (declarch-variant.kdl)
    ///   - user/repo/branch       Fetch from specific branch
    ///   - gitlab.com/user/repo   Fetch from GitLab
    ///   - https://...            Direct URL to declarch.kdl
    ///   - hyprland/niri-nico     Fetch from official registry
    Init {
        /// Config source (GitHub/GitLab repo, URL, or registry module)
        ///
        /// Examples:
        ///   exampleuser/hyprland1           GitHub: user/repo (fetches declarch.kdl)
        ///   exampleuser/dotfiles:uwsm       GitHub: user/repo:variant (fetches declarch-uwsm.kdl)
        ///   exampleuser/dotfiles:minimal    GitHub: user/repo:variant (fetches declarch-minimal.kdl)
        ///   hyprwm/hyprland             GitHub: official project config
        ///   exampleuser/hyprland1/develop   GitHub: user/repo/branch
        ///   gitlab.com/user/repo         GitLab repository
        ///   https://example.com/config.kdl  Direct URL
        ///   hyprland/niri-nico           Official registry
        #[arg(value_name = "SOURCE")]
        path: Option<String>,

        /// Hostname specific config
        #[arg(long)]
        host: Option<String>,

        /// Skip automatic Soar installation
        #[arg(long, help_heading = "Advanced")]
        skip_soar_install: bool,
    },

    /// Synchronize system state with configuration
    Sync {
        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,

        /// Remove packages not defined in config (Strict Mode)
        #[arg(long)]
        prune: bool,

        /// Update system (paru -Syu) before syncing
        #[arg(short = 'u', long)]
        update: bool,

        /// Garbage collect system orphans after sync
        #[arg(long)]
        gc: bool,

        // --- Advanced Options Group ---

        /// Sync only specific package or scope (e.g. "firefox", "aur", "flatpak")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Skip automatic Soar installation
        #[arg(long, help_heading = "Advanced")]
        skip_soar_install: bool,
    },

    /// Verify configuration syntax and imports
    Check {
        /// Verbose output (list resolved packages)
        #[arg(long)]
        verbose: bool,

        /// Check for duplicate package declarations
        #[arg(long)]
        duplicates: bool,

        /// Check for cross-backend package name conflicts
        #[arg(long)]
        conflicts: bool,
    },

    /// Show system status and managed packages
    Info,

    /// Switch package variant (e.g., hyprland -> hyprland-git)
    Switch {
        /// Old package name to remove
        #[arg(value_name = "OLD_PACKAGE")]
        old_package: String,

        /// New package name to install
        #[arg(value_name = "NEW_PACKAGE")]
        new_package: String,

        /// Backend (aur, flatpak, or soar)
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Dry run - show what would happen
        #[arg(long)]
        dry_run: bool,
    },

    /// Edit configuration files
    Edit {
        /// Module or config to edit (optional)
        /// If not provided, edits root declarch.kdl
        /// If provided, edits specific module (e.g., "hyprland/niri-nico")
        #[arg(value_name = "TARGET")]
        target: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
