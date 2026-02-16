use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(
    name = "declarch",
    about = "Universal declarative package manager - unify aur, flatpak, npm, nix, cargo, pip, and custom backends under one declarative config(s).",
    version,
    help_template = "{about-with-newline}
{usage-heading} {usage}

Commands:
{subcommands}

Options:
{options}
{after-help}
",
    term_width = 100,
    max_term_width = 120
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

    /// Preview changes without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Output format (table, json, yaml)
    #[arg(long, value_name = "FORMAT", global = true)]
    pub format: Option<String>,

    /// Machine output contract version (placeholder, e.g. v1)
    #[arg(long, value_name = "VERSION", global = true)]
    pub output_version: Option<String>,
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

        /// Create new backend configuration file(s)
        ///
        /// Creates backend definition files in ~/.config/declarch/backends/
        /// Supports multiple backends via multiple flags
        ///
        /// Examples:
        ///   declarch init --backend cargo
        ///   declarch init --backend apt --backend cargo
        ///   declarch init --backend apt --backend cargo -y
        #[arg(
            long,
            value_name = "NAME",
            group = "init_target",
            num_args = 1..,
            value_delimiter = ','
        )]
        backend: Vec<String>,

        /// List available modules/backends from registry
        ///
        /// Shows all available items in the declarch-packages registry
        #[arg(long, value_name = "WHAT")]
        list: Option<String>,

        /// Create local module (skip registry lookup)
        ///
        /// Bypasses registry lookup and creates a local module directly
        #[arg(long, group = "init_target")]
        local: bool,

        /// Restore backends.kdl from template (overwrite existing)
        #[arg(long, group = "restore")]
        restore_backends: bool,

        /// Restore declarch.kdl from template (overwrite existing)  
        #[arg(long, group = "restore")]
        restore_declarch: bool,
    },

    /// Synchronize system state with configuration
    ///
    /// Main command for managing packages. Use global `--dry-run` for preview mode.
    /// Subcommands provide additional functionality like updating package indices,
    /// upgrading packages, and cleaning caches.
    #[command(after_help = "Most common flow:
  declarch --dry-run sync
  declarch sync

Other useful commands:
  declarch sync update
  declarch sync prune")]
    Sync {
        /// Sync only specific package or scope (e.g. "firefox", "backend-name")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Show diff before syncing (like git diff)
        #[arg(long, help_heading = "Advanced")]
        diff: bool,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Activate optional profile block from config (e.g. profile "desktop" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        profile: Option<String>,

        /// Activate optional host block from config (e.g. host "vps-1" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        host: Option<String>,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,

        #[command(subcommand)]
        command: Option<SyncCommand>,

        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,
    },

    /// Show status, diagnosis, and package reasoning
    Info {
        /// Query a package/backend/module ("why is this here?")
        #[arg(value_name = "QUERY")]
        query: Option<String>,

        /// Run diagnostics
        #[arg(long)]
        doctor: bool,

        /// Show sync plan reasoning (install/remove drift)
        #[arg(long)]
        plan: bool,

        /// List managed packages (same as old `info list`)
        #[arg(long)]
        list: bool,

        /// With --list: show orphan packages only
        #[arg(long)]
        orphans: bool,

        /// With --list: show synced packages only
        #[arg(long)]
        synced: bool,

        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Filter by package name (status mode)
        #[arg(long, value_name = "PACKAGE")]
        package: Option<String>,

        /// Activate optional profile block (profile "NAME" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        profile: Option<String>,

        /// Activate optional host block (host "NAME" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        host: Option<String>,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

    /// Switch package variant (e.g., hyprland -> hyprland-git)
    Switch {
        /// Old package name to remove
        #[arg(value_name = "OLD_PACKAGE")]
        old_package: String,

        /// New package name to install
        #[arg(value_name = "NEW_PACKAGE")]
        new_package: String,

        /// Backend (e.g., system package manager, container runtime, language tool)
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,
    },

    /// Edit configuration files
    Edit {
        /// Module or config to edit (optional)
        /// If not provided, edits root declarch.kdl
        /// If provided, edits specific module (e.g., "hyprland/niri-nico")
        #[arg(value_name = "TARGET")]
        target: Option<String>,

        /// Preview content without opening editor (like cat)
        #[arg(long, short)]
        preview: bool,

        /// Show line numbers in preview
        #[arg(long, requires = "preview")]
        number: bool,

        /// Create new module from template if it doesn't exist
        #[arg(long, short)]
        create: bool,

        /// Auto-format KDL before opening
        #[arg(long)]
        auto_format: bool,

        /// Only validate syntax, don't open editor (exit 0/1)
        #[arg(long)]
        validate_only: bool,

        /// Create backup before editing (.backup suffix)
        #[arg(long, short)]
        backup: bool,
    },

    /// Install packages to configuration
    ///
    /// Adds packages to KDL configuration files and automatically syncs the system.
    ///
    /// Common examples:
    /// - declarch install aur:hyprland
    /// - declarch install aur:vim aur:nano aur:emacs
    /// - declarch install soar:bat
    /// - declarch install bat fzf ripgrep --backend aur
    /// - declarch install npm:typescript --module development
    #[command(verbatim_doc_comment)]
    Install {
        /// Package(s) to install (format: [backend:]package)
        ///
        /// Examples:
        /// - hyprland
        /// - backend:package
        /// - backend:name
        #[arg(required = true, num_args = 1.., value_name = "PACKAGES")]
        packages: Vec<String>,

        /// Target package manager for all packages
        ///
        /// If specified, all packages will use this backend unless overridden
        /// by the backend:package syntax in individual package names.
        #[arg(short = 'b', long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Target module file (e.g., "base" or "linux/notes")
        ///
        /// If not specified, packages are added to modules/others.kdl
        #[arg(short = 'm', long, value_name = "MODULE")]
        module: Option<String>,

        /// Don't sync after editing config
        #[arg(long)]
        no_sync: bool,
    },

    /// Search for packages across backends
    ///
    /// Search for packages across all configured backends.
    ///
    /// Examples:
    ///   declarch search firefox                Search for firefox in all backends
    ///   declarch search firefox --backends aur   Search in specific backend only
    ///   declarch search bat --installed-only   Show only installed matches
    ///   declarch search backend:package        Search in specific backend (alternative syntax)
    ///   declarch search firefox --local        Search only in installed packages
    Search {
        /// Search query (can use "backend:query" syntax for specific backend)
        #[arg(value_name = "QUERY")]
        query: String,

        /// Filter by backends (comma-separated or multiple flags)
        #[arg(
            short = 'b',
            long,
            value_name = "BACKENDS",
            value_delimiter = ',',
            help_heading = "Filtering"
        )]
        backends: Vec<String>,

        /// Limit results per backend (default: 10, use "all" or 0 for unlimited)
        #[arg(long, value_name = "NUM", help_heading = "Filtering")]
        limit: Option<String>,

        /// Show only installed packages
        #[arg(long, help_heading = "Filtering")]
        installed_only: bool,

        /// Show only available packages (not installed)
        #[arg(long, help_heading = "Filtering")]
        available_only: bool,

        /// Search only in locally installed packages (uses search_local_cmd)
        #[arg(long, help_heading = "Filtering")]
        local: bool,
    },

    /// Lint configuration quality with beginner-friendly checks
    Lint {
        /// Treat warnings as errors (exit non-zero)
        #[arg(long)]
        strict: bool,

        /// Apply safe automatic fixes (currently import sorting/format cleanup)
        #[arg(long)]
        fix: bool,

        /// Lint scope: all, validate, duplicates, conflicts
        #[arg(long, value_enum, default_value_t = LintMode::All)]
        mode: LintMode,

        /// Filter by backend for package-level checks
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Show planned install/remove drift (like old check --diff)
        #[arg(long)]
        diff: bool,

        /// Show timing information
        #[arg(long)]
        benchmark: bool,

        /// Repair malformed/corrupt package entries in state.json
        #[arg(long, help_heading = "Advanced")]
        repair_state: bool,

        /// Activate optional profile block (profile \"NAME\" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        profile: Option<String>,

        /// Activate optional host block (host \"NAME\" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        host: Option<String>,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

    /// Generate shell completions (hidden from main help)
    #[command(hide = true)]
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Extension protocol placeholder (hidden)
    #[command(hide = true)]
    Ext,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LintMode {
    All,
    Validate,
    Duplicates,
    Conflicts,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SyncCommand {
    /// Sync with system update
    ///
    /// Runs system package manager update before syncing packages.
    Update {
        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,

        /// Sync only specific package or scope (e.g. "firefox", "backend-name")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Show diff before syncing (like git diff)
        #[arg(long, help_heading = "Advanced")]
        diff: bool,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Activate optional profile block from config (e.g. profile "desktop" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        profile: Option<String>,

        /// Activate optional host block from config (e.g. host "vps-1" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        host: Option<String>,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

    /// Sync with package removal
    ///
    /// Removes packages that are not defined in your configuration (Strict Mode).
    Prune {
        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,

        /// Sync only specific package or scope (e.g. "firefox", "backend-name")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Show diff before syncing (like git diff)
        #[arg(long, help_heading = "Advanced")]
        diff: bool,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Activate optional profile block from config (e.g. profile "desktop" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        profile: Option<String>,

        /// Activate optional host block from config (e.g. host "vps-1" { ... })
        #[arg(long, value_name = "NAME", help_heading = "Targeting")]
        host: Option<String>,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

    /// Clean package manager caches
    ///
    /// Removes cached package files for configured backends.
    /// Useful for freeing disk space or resolving cache corruption issues.
    ///
    /// Examples:
    ///   declarch sync cache              Clean all backend caches
    ///   declarch sync cache --backend npm  Clean only npm cache
    Cache {
        /// Target specific backend(s)
        #[arg(short, long, value_name = "BACKEND")]
        backend: Vec<String>,
    },

    /// Upgrade packages to latest versions
    ///
    /// Upgrades all packages managed by declarch to their latest versions
    /// across all configured backends. After upgrading, automatically runs
    /// sync to adopt the new versions into state.
    ///
    /// Examples:
    ///   declarch sync upgrade              Upgrade all packages
    ///   declarch sync upgrade --backend npm  Upgrade only npm packages
    ///   declarch sync upgrade --no-sync    Upgrade without auto-sync
    Upgrade {
        /// Target specific backend(s)
        #[arg(short, long, value_name = "BACKEND")]
        backend: Vec<String>,

        /// Skip automatic sync after upgrade
        #[arg(long)]
        no_sync: bool,
    },
}
