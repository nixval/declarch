use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct RawConfig {
    // === Existing fields ===
    pub imports: Vec<String>,
    /// Packages from AUR (Arch Linux specific)
    /// Syntax: packages { ... } or packages:aur { ... }
    pub packages: Vec<PackageEntry>,
    /// Packages from Soar registry (cross-distro static binaries)
    /// Syntax: packages:soar { ... } or soar:package in packages block
    pub soar_packages: Vec<PackageEntry>,
    /// Flatpak packages
    /// Syntax: packages:flatpak { ... } or flatpak:package in packages block
    pub flatpak_packages: Vec<PackageEntry>,

    // === New language ecosystem backends ===
    /// npm packages (Node.js global packages)
    /// Syntax: packages:npm { ... } or npm:package in packages block
    pub npm_packages: Vec<PackageEntry>,
    /// Yarn packages
    /// Syntax: packages:yarn { ... }
    pub yarn_packages: Vec<PackageEntry>,
    /// pnpm packages
    /// Syntax: packages:pnpm { ... }
    pub pnpm_packages: Vec<PackageEntry>,
    /// Bun packages
    /// Syntax: packages:bun { ... }
    pub bun_packages: Vec<PackageEntry>,
    /// pip packages (Python)
    /// Syntax: packages:pip { ... }
    pub pip_packages: Vec<PackageEntry>,
    /// Cargo packages (Rust)
    /// Syntax: packages:cargo { ... }
    pub cargo_packages: Vec<PackageEntry>,
    /// Homebrew packages
    /// Syntax: packages:brew { ... }
    pub brew_packages: Vec<PackageEntry>,

    /// Custom backend packages (user-defined)
    /// Syntax: packages:nala { ... } where nala is defined in backends.kdl
    /// Stored as HashMap: backend_name -> Vec<PackageEntry>
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,

    pub excludes: Vec<String>,
    /// Package mappings: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub package_mappings: HashMap<String, String>,
    /// Editor to use for edit command
    /// Syntax: editor "nvim" or editor nvim
    pub editor: Option<String>,

    // === NEW: Meta block ===
    /// Configuration metadata
    pub meta: ConfigMeta,

    // === NEW: Conflicts ===
    /// Mutually exclusive packages
    pub conflicts: Vec<ConflictEntry>,

    // === NEW: Backend options ===
    /// Backend-specific configuration options
    /// Syntax: options:aur { noconfirm true }
    pub backend_options: HashMap<String, HashMap<String, String>>,

    // === NEW: Environment variables ===
    /// Environment variables for package operations
    /// Syntax: env { "EDITOR=nvim" } or env:aur { "MAKEFLAGS=-j4" }
    pub env: HashMap<String, Vec<String>>,

    // === NEW: Package repositories ===
    /// Custom package repositories
    /// Syntax: repos:aur { "https://..." }
    pub repositories: HashMap<String, Vec<String>>,

    // === NEW: Policy control ===
    /// Package lifecycle policies
    pub policy: PolicyConfig,

    // === NEW: Hooks ===
    /// Pre/post sync hooks
    pub hooks: HookConfig,
}

/// Package entry (version constraints skipped for now)
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
}

/// Configuration metadata
#[derive(Debug, Clone, Default)]
pub struct ConfigMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

/// Conflict entry - mutually exclusive packages
#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub packages: Vec<String>,
    pub condition: Option<String>, // Future: for conditional conflicts
}

/// Package lifecycle policies
#[derive(Debug, Clone, Default)]
pub struct PolicyConfig {
    /// Protected packages that won't be removed even with --prune
    pub protected: HashSet<String>,
    /// Strategy for handling orphans: "keep", "remove", "ask"
    pub orphans: Option<String>,
}

/// Hook configuration
#[derive(Debug, Clone, Default)]
pub struct HookConfig {
    /// All hooks (organized by phase during execution)
    pub hooks: Vec<HookEntry>,
}

/// Hook entry
#[derive(Debug, Clone)]
pub struct HookEntry {
    pub command: String,
    pub hook_type: HookType,
    pub phase: HookPhase,
    pub package: Option<String>,
    pub conditions: Vec<HookCondition>,
    pub error_behavior: ErrorBehavior,
}

/// Hook type (simplified from v0.4.3)
#[derive(Debug, Clone, PartialEq)]
pub enum HookType {
    User, // Run without sudo
    Root, // Run with sudo
}

/// Hook phase - when the hook should run
#[derive(Debug, Clone, PartialEq)]
pub enum HookPhase {
    // Global phases
    PreSync,
    PostSync,
    OnSuccess,
    OnFailure,
    // Package phases
    PreInstall,
    PostInstall,
    PreRemove,
    PostRemove,
    OnUpdate,
}

/// Hook condition - when to run the hook
#[derive(Debug, Clone, PartialEq)]
pub enum HookCondition {
    IfInstalled(String),  // Run only if package is installed
    IfChanged(String),    // Run only if package was installed/updated
    IfBackend(String),    // Run only if this backend had changes
    IfSuccess,            // Run only if previous hook succeeded
}

/// Error behavior for hooks
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorBehavior {
    Warn,     // Default: warn on error (from v0.4.3)
    Required, // Fail sync if hook fails (new in v0.4.4)
    Ignore,   // Silently ignore errors (new in v0.4.4)
}

impl Default for ErrorBehavior {
    fn default() -> Self {
        Self::Warn
    }
}
