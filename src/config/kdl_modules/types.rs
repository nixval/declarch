use std::collections::{HashMap, HashSet};

/// Empty vector constant for default returns
static EMPTY_VEC: Vec<PackageEntry> = Vec::new();

#[derive(Debug, Clone)]
pub struct RawConfig {
    pub imports: Vec<String>,

    /// All built-in backend packages stored in a single HashMap
    /// Key: Backend enum, Value: Vector of package entries
    /// This replaces the individual *_packages fields
    pub packages: HashMap<Backend, Vec<PackageEntry>>,

    /// Custom backend packages (user-defined)
    /// Syntax: packages:nala { ... } where nala is defined in backends.kdl
    /// Stored as HashMap: backend_name -> Vec<PackageEntry>
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,

    pub excludes: Vec<String>,
    /// Package mappings: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub package_mappings: HashMap<String, String>,

    /// Project metadata
    pub project_metadata: ProjectMetadata,

    /// Mutually exclusive packages
    pub conflicts: Vec<ConflictEntry>,

    /// Backend-specific configuration options
    /// Syntax: options:aur { noconfirm true }
    pub backend_options: HashMap<String, HashMap<String, String>>,

    /// Environment variables for package operations
    /// Syntax: env { "EDITOR=nvim" } or env:aur { "MAKEFLAGS=-j4" }
    pub env: HashMap<String, Vec<String>>,

    /// Custom package sources
    /// Syntax: repos:aur { "https://..." }
    pub package_sources: HashMap<String, Vec<String>>,

    /// Package lifecycle policies
    pub policy: PolicyConfig,

    /// Pre/post sync lifecycle actions
    pub lifecycle_actions: LifecycleConfig,
}

impl RawConfig {
    /// Create a new empty RawConfig with all backends initialized
    pub fn new() -> Self {
        let mut packages = HashMap::new();
        packages.insert(Backend::Aur, Vec::new());
        packages.insert(Backend::Flatpak, Vec::new());
        packages.insert(Backend::Soar, Vec::new());
        packages.insert(Backend::Npm, Vec::new());
        packages.insert(Backend::Yarn, Vec::new());
        packages.insert(Backend::Pnpm, Vec::new());
        packages.insert(Backend::Bun, Vec::new());
        packages.insert(Backend::Pip, Vec::new());
        packages.insert(Backend::Cargo, Vec::new());
        packages.insert(Backend::Brew, Vec::new());

        Self {
            imports: Vec::new(),
            packages,
            custom_packages: HashMap::new(),
            excludes: Vec::new(),
            package_mappings: HashMap::new(),
            project_metadata: ProjectMetadata::default(),
            conflicts: Vec::new(),
            backend_options: HashMap::new(),
            env: HashMap::new(),
            package_sources: HashMap::new(),
            policy: PolicyConfig::default(),
            lifecycle_actions: LifecycleConfig::default(),
        }
    }

    // === Backward compatibility accessors ===

    /// Get AUR packages (backward compatibility)
    pub fn aur_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Aur).unwrap_or(&EMPTY_VEC)
    }

    /// Get Flatpak packages (backward compatibility)
    pub fn flatpak_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Flatpak).unwrap_or(&EMPTY_VEC)
    }

    /// Get Soar packages (backward compatibility)
    pub fn soar_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Soar).unwrap_or(&EMPTY_VEC)
    }

    /// Get npm packages (backward compatibility)
    pub fn npm_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Npm).unwrap_or(&EMPTY_VEC)
    }

    /// Get Yarn packages (backward compatibility)
    pub fn yarn_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Yarn).unwrap_or(&EMPTY_VEC)
    }

    /// Get pnpm packages (backward compatibility)
    pub fn pnpm_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Pnpm).unwrap_or(&EMPTY_VEC)
    }

    /// Get Bun packages (backward compatibility)
    pub fn bun_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Bun).unwrap_or(&EMPTY_VEC)
    }

    /// Get pip packages (backward compatibility)
    pub fn pip_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Pip).unwrap_or(&EMPTY_VEC)
    }

    /// Get Cargo packages (backward compatibility)
    pub fn cargo_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Cargo).unwrap_or(&EMPTY_VEC)
    }

    /// Get Homebrew packages (backward compatibility)
    pub fn brew_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Brew).unwrap_or(&EMPTY_VEC)
    }

    /// Get packages for a specific backend
    pub fn packages_for(&self, backend: &Backend) -> &Vec<PackageEntry> {
        self.packages.get(backend).unwrap_or(&EMPTY_VEC)
    }

    /// Get mutable packages for a specific backend
    pub fn packages_for_mut(&mut self, backend: &Backend) -> &mut Vec<PackageEntry> {
        self.packages.entry(backend.clone()).or_default()
    }

    /// Add packages to a specific backend
    pub fn add_packages(&mut self, backend: &Backend, entries: Vec<PackageEntry>) {
        self.packages
            .entry(backend.clone())
            .or_default()
            .extend(entries);
    }

    /// Iterate over all packages grouped by backend
    pub fn all_packages(&self) -> impl Iterator<Item = (&Backend, &Vec<PackageEntry>)> {
        self.packages.iter()
    }

    /// Check if there are any packages configured
    pub fn has_packages(&self) -> bool {
        self.packages.values().any(|v| !v.is_empty()) || !self.custom_packages.is_empty()
    }

    /// Get total package count across all backends
    pub fn total_package_count(&self) -> usize {
        let builtin: usize = self.packages.values().map(|v| v.len()).sum();
        let custom: usize = self.custom_packages.values().map(|v| v.len()).sum();
        builtin + custom
    }
}

impl Default for RawConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Backend enum imported from core::types
pub use crate::core::types::Backend;

/// Package entry (version constraints skipped for now)
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
}

/// Project metadata
#[derive(Debug, Clone, Default)]
pub struct ProjectMetadata {
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

/// Lifecycle action configuration
#[derive(Debug, Clone, Default)]
pub struct LifecycleConfig {
    /// All lifecycle actions (organized by phase during execution)
    pub actions: Vec<LifecycleAction>,
}

/// Lifecycle action entry
#[derive(Debug, Clone)]
pub struct LifecycleAction {
    pub command: String,
    pub action_type: ActionType,
    pub phase: LifecyclePhase,
    pub package: Option<String>,
    pub conditions: Vec<ActionCondition>,
    pub error_behavior: ErrorBehavior,
}

/// Action type (simplified from v0.4.3)
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    User, // Run without sudo
    Root, // Run with sudo
}

/// Lifecycle phase - when the action should run
#[derive(Debug, Clone, PartialEq)]
pub enum LifecyclePhase {
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

/// Action condition - when to run the action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionCondition {
    IfInstalled(String), // Run only if package is installed
    IfChanged(String),   // Run only if package was installed/updated
    IfBackend(String),   // Run only if this backend had changes
    IfSuccess,           // Run only if previous action succeeded
}

/// Error behavior for hooks
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ErrorBehavior {
    #[default]
    Warn, // Default: warn on error (from v0.4.3)
    Required, // Fail sync if hook fails (new in v0.4.4)
    Ignore,   // Silently ignore errors (new in v0.4.4)
}
