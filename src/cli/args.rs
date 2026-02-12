use clap::{Parser, Subcommand};
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
        #[arg(long, value_name = "NAME", group = "init_target", num_args = 1..)]
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

    },

    /// Synchronize system state with configuration
    ///
    /// Main command for managing packages. Subcommands provide additional
    /// functionality like previewing changes, updating package indices,
    /// upgrading packages, and cleaning caches.
    Sync {
        #[command(subcommand)]
        command: Option<SyncCommand>,

        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,
    },

    /// Verify configuration syntax and imports
    Check {
        #[command(subcommand)]
        command: Option<CheckCommand>,

        /// [DEPRECATED] Use subcommand instead (e.g., `declarch check duplicates`)
        #[arg(long, hide = true)]
        duplicates: bool,

        /// [DEPRECATED] Use subcommand instead (e.g., `declarch check conflicts`)
        #[arg(long, hide = true)]
        conflicts: bool,

        /// [DEPRECATED] Use `declarch check duplicates` instead
        #[arg(long, hide = true)]
        only_duplicates: bool,

        /// [DEPRECATED] Use `declarch check conflicts` instead
        #[arg(long, hide = true)]
        only_conflicts: bool,

        /// [DEPRECATED] Use `declarch check validate` instead
        #[arg(long, hide = true)]
        validate: bool,
    },

    /// Show system status and managed packages
    Info {
        #[command(subcommand)]
        command: Option<InfoCommand>,

        /// [DEPRECATED] Use `declarch info doctor` instead
        #[arg(long, hide = true)]
        doctor: bool,
    },

    /// List installed packages
    List {
        #[command(subcommand)]
        command: Option<ListCommand>,

        /// [DEPRECATED] Use `declarch list orphans` instead
        #[arg(long, hide = true)]
        orphans: bool,

        /// [DEPRECATED] Use `declarch list synced` instead
        #[arg(long, hide = true)]
        synced: bool,
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

    /// Install packages to configuration
    ///
    /// Adds packages to KDL configuration files and automatically syncs the system.
    ///
    /// Examples:
    ///   declarch install hyprland              Add to modules/others.kdl
    ///   declarch install vim nano emacs        Add multiple packages
    ///   declarch install soar:bat              Add to backend-specific block
    ///   declarch install package --modules base    Add to specific module
    Install {
        /// Package(s) to install (format: [backend:]package)
        ///
        /// Examples:
        ///   hyprland                     Package without backend (uses default)
        ///   backend:package              Package with backend override
        ///   backend:name                 Package with specific backend
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

    /// Manage declarch settings
    ///
    /// Configure output format, colors, and other preferences.
    ///
    /// Examples:
    ///   declarch settings set color never     Disable colors
    ///   declarch settings set format json     Set output format to JSON
    ///   declarch settings show                Show all settings
    Settings {
        #[command(subcommand)]
        command: SettingsCommand,
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
        #[arg(short = 'b', long, value_name = "BACKENDS", value_delimiter = ',', help_heading = "Filtering")]
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

    /// Generate shell completions (hidden from main help)
    #[command(hide = true)]
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SyncCommand {
    /// Full sync (default)
    ///
    /// Synchronizes packages without updating the system or removing packages.
    /// Use `update` to run system updates, or `prune` to remove undefined packages.
    Sync {
        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,

        /// Sync only specific package or scope (e.g. "firefox", "backend-name")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

    /// Preview changes without executing
    ///
    /// Shows what would be installed, updated, or removed without making changes.
    Preview {
        /// Garbage collect system orphans after sync
        #[arg(long, help_heading = "Advanced")]
        gc: bool,

        /// Sync only specific package or scope (e.g. "firefox", "backend-name")
        #[arg(long, value_name = "TARGET", help_heading = "Targeting")]
        target: Option<String>,

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES", help_heading = "Advanced")]
        modules: Vec<String>,
    },

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

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

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

        /// Skip package manager confirmation prompts (CI/CD)
        #[arg(long, help_heading = "Advanced")]
        noconfirm: bool,

        /// Enable hooks (disabled by default for security)
        #[arg(long, help_heading = "Advanced")]
        hooks: bool,

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

#[derive(Subcommand, Debug, Clone)]
pub enum InfoCommand {
    /// Show system status (default)
    ///
    /// Displays information about managed packages and system state.
    /// This is the default behavior when no subcommand is specified.
    Status {
        /// Enable verbose logging
        #[arg(long)]
        debug: bool,

        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Filter by package name
        #[arg(long, value_name = "PACKAGE")]
        package: Option<String>,
    },

    /// Diagnose system issues
    ///
    /// Runs diagnostic checks to identify configuration issues,
    /// missing dependencies, and other system problems.
    Doctor {
        /// Enable verbose logging
        #[arg(long)]
        debug: bool,

        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Filter by package name
        #[arg(long, value_name = "PACKAGE")]
        package: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ListCommand {
    /// List all packages (default)
    ///
    /// Lists all installed packages managed by declarch.
    /// This is the default behavior when no subcommand is specified.
    All {
        /// Filter by backend name
        #[arg(short, long, value_name = "BACKEND")]
        backend: Option<String>,
    },

    /// List orphan packages
    ///
    /// Lists packages that are installed on the system but not
    /// defined in your declarch configuration.
    Orphans {
        /// Filter by backend name
        #[arg(short, long, value_name = "BACKEND")]
        backend: Option<String>,
    },

    /// List synced packages
    ///
    /// Lists packages that are both defined in your configuration
    /// and currently installed on the system.
    Synced {
        /// Filter by backend name
        #[arg(short, long, value_name = "BACKEND")]
        backend: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CheckCommand {
    /// Run all checks (default)
    ///
    /// Checks configuration syntax, imports, duplicates, and conflicts.
    /// This is the default behavior when no subcommand is specified.
    All {
        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Show planned changes without executing
        #[arg(long)]
        diff: bool,

        /// Show performance metrics
        #[arg(long)]
        benchmark: bool,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES")]
        modules: Vec<String>,
    },

    /// Check for duplicate package declarations
    ///
    /// Finds packages that are declared multiple times across your configuration.
    /// Duplicates are automatically deduplicated during sync.
    Duplicates {
        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Show planned changes without executing
        #[arg(long)]
        diff: bool,
    },

    /// Check for cross-backend package name conflicts
    ///
    /// Finds packages with the same name in different backends.
    /// This can cause PATH conflicts when multiple backends install binaries with the same name.
    Conflicts {
        /// Filter by backend name
        #[arg(long, value_name = "BACKEND")]
        backend: Option<String>,

        /// Show planned changes without executing
        #[arg(long)]
        diff: bool,
    },

    /// Validate syntax only
    ///
    /// Checks configuration file syntax and imports without checking for duplicates or conflicts.
    Validate {
        /// Show performance metrics
        #[arg(long)]
        benchmark: bool,

        /// Load additional modules temporarily
        #[arg(long, value_name = "MODULES")]
        modules: Vec<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SettingsCommand {
    /// Set a setting value
    ///
    /// Examples:
    ///   declarch settings set color never
    ///   declarch settings set format json
    Set {
        /// Setting name (color, progress, format, verbose)
        #[arg(value_name = "KEY")]
        key: String,

        /// Setting value
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Get a setting value
    ///
    /// Example:
    ///   declarch settings get color
    Get {
        /// Setting name
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Show all settings
    ///
    /// Example:
    ///   declarch settings show
    Show,

    /// Reset setting to default
    ///
    /// Example:
    ///   declarch settings reset color
    Reset {
        /// Setting name
        #[arg(value_name = "KEY")]
        key: String,
    },
}
