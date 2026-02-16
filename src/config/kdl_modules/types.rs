use std::collections::{HashMap, HashSet};

/// Raw configuration parsed from KDL files
///
/// In v0.6+, this uses unified package storage where all packages are
/// organized by backend name in a HashMap. No backend-specific fields.
#[derive(Debug, Clone, Default)]
pub struct RawConfig {
    /// Import statements
    pub imports: Vec<String>,

    /// Preferred editor for 'declarch edit' command
    /// Priority: $VISUAL env → $EDITOR env → this field → "nano"
    pub editor: Option<String>,

    /// Unified package storage: backend_name -> packages
    ///
    /// All packages are stored here, organized by their backend.
    /// Examples:
    /// - "paru" -> [hyprland, waybar, git]
    /// - "npm" -> [typescript, eslint]
    /// - "flatpak" -> [com.spotify.Client]
    ///
    /// Syntax in KDL:
    ///   pkg { paru { hyprland waybar } }
    ///   pkg:paru { hyprland waybar }
    pub packages_by_backend: HashMap<String, Vec<PackageEntry>>,

    /// Packages to exclude from sync
    pub excludes: Vec<String>,

    /// Project metadata
    pub project_metadata: ProjectMetadata,

    /// Mutually exclusive packages
    pub conflicts: Vec<ConflictEntry>,

    /// Backend-specific configuration options
    pub backend_options: HashMap<String, HashMap<String, String>>,

    /// Environment variables for package operations
    pub env: HashMap<String, Vec<String>>,

    /// Custom package sources (repositories)
    pub package_sources: HashMap<String, Vec<String>>,

    /// Package lifecycle policies
    pub policy: PolicyConfig,

    /// Pre/post sync lifecycle actions
    pub lifecycle_actions: LifecycleConfig,

    /// Backend definition imports (paths to backend files)
    /// These define which backends are available for package operations
    pub backend_imports: Vec<String>,

    /// Experimental feature flags.
    /// Example:
    /// experimental {
    ///   "enable-hooks"
    /// }
    pub experimental: Vec<String>,

    /// MCP access policy.
    /// Example:
    /// mcp {
    ///   mode "read-only"
    ///   allow_tools "declarch_sync_apply"
    /// }
    pub mcp: McpConfig,
}

/// Package entry (minimal - just name for now)
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
    pub maintainers: Vec<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub platforms: Vec<String>,
    pub requires: Vec<String>,
    pub installation_guide: Option<String>,
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
    /// Require explicit backend declarations (no implicit/legacy default backend)
    pub require_backend: Option<bool>,
    /// Disallow hook execution even when CLI uses --hooks
    pub forbid_hooks: Option<bool>,
    /// Duplicate declaration policy: "warn" | "error"
    pub on_duplicate: Option<String>,
    /// Cross-backend conflict policy: "warn" | "error"
    pub on_conflict: Option<String>,
}

impl PolicyConfig {
    pub fn duplicate_is_error(&self) -> bool {
        matches!(self.on_duplicate.as_deref(), Some("error"))
    }

    pub fn conflict_is_error(&self) -> bool {
        matches!(self.on_conflict.as_deref(), Some("error"))
    }
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

/// Action type
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    User, // Run without sudo
    Root, // Run with sudo
}

/// Lifecycle phase - when the action should run
#[derive(Debug, Clone, PartialEq)]
pub enum LifecyclePhase {
    PreSync,
    PostSync,
    OnSuccess,
    OnFailure,
    PreInstall,
    PostInstall,
    PreRemove,
    PostRemove,
    OnUpdate,
}

/// Action condition - when to run the action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionCondition {
    IfInstalled(String),
    IfChanged(String),
    IfBackend(String),
    IfSuccess,
}

/// Error behavior for hooks
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ErrorBehavior {
    #[default]
    Warn,
    Required,
    Ignore,
}

/// MCP policy configuration.
#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    /// "read-only" (default) or "write-enabled"
    pub mode: Option<String>,
    /// Explicitly allowed tool names for write operations.
    pub allow_tools: Vec<String>,
}
