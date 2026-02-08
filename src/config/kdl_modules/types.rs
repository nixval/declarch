use std::collections::{HashMap, HashSet};

/// Raw configuration parsed from KDL files
/// 
/// In v0.6+, this uses unified package storage where all packages are
/// organized by backend name in a HashMap. No backend-specific fields.
#[derive(Debug, Clone)]
pub struct RawConfig {
    /// Import statements
    pub imports: Vec<String>,
    
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
    
    /// Package mappings: config_name -> actual_package_name
    pub package_mappings: HashMap<String, String>,

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
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            imports: Vec::new(),
            packages_by_backend: HashMap::new(),
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
