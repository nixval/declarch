use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Re-export Backend from core to avoid duplication
pub use crate::core::types::Backend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub meta: StateMeta,
    // Key format: "backend:package_name"
    pub packages: HashMap<String, PackageState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMeta {
    pub schema_version: u8,
    pub last_sync: DateTime<Utc>,
    pub hostname: String,

    // #[serde(default)] ensures it loads as None for existing state.json
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,

    /// Monotonic revision for state writes (optional for backward compatibility)
    #[serde(default)]
    pub state_revision: Option<u64>,

    /// Generator/build metadata for debugging migrations
    #[serde(default)]
    pub generator: Option<String>,

    /// Optional feature flags written to state (non-breaking extension point)
    #[serde(default)]
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageState {
    pub backend: Backend,

    /// Name from config file (what user wrote)
    #[serde(default)]
    pub config_name: String,

    /// Name that `pacman -Q` shows (provided name)
    /// This is the primary identity for matching
    #[serde(default)]
    pub provides_name: String,

    /// Actual system package name (if different from provides_name)
    /// None for flatpak/npm/etc or when actual == provides_name
    /// Previously called aur_package_name for backward compatibility
    #[serde(default)]
    #[serde(alias = "aur_package_name")]
    pub actual_package_name: Option<String>,

    pub installed_at: DateTime<Utc>,
    pub version: Option<String>,

    /// How this package entered state: declared/adopted/manual-sync
    #[serde(default)]
    pub install_reason: Option<String>,

    /// Module path that declared this package (if known)
    #[serde(default)]
    pub source_module: Option<String>,

    /// Last time this package was observed in installed snapshot
    #[serde(default)]
    pub last_seen_at: Option<DateTime<Utc>>,

    /// Backend-specific extension payload for future features.
    #[serde(default)]
    pub backend_meta: Option<serde_json::Value>,
}

// Implement Default for PackageState to help with serde defaults
impl Default for PackageState {
    fn default() -> Self {
        Self {
            backend: Backend::from("unknown"),
            config_name: String::new(),
            provides_name: String::new(),
            actual_package_name: None,
            installed_at: Utc::now(),
            version: None,
            install_reason: None,
            source_module: None,
            last_seen_at: None,
            backend_meta: None,
        }
    }
}

impl PackageState {
    /// Create new PackageState from config name
    pub fn from_config(config_name: String, backend: Backend, version: Option<String>) -> Self {
        Self {
            backend,
            config_name: config_name.clone(),
            provides_name: config_name,
            actual_package_name: None,
            installed_at: Utc::now(),
            version,
            install_reason: Some("declared".to_string()),
            source_module: None,
            last_seen_at: Some(Utc::now()),
            backend_meta: None,
        }
    }

    /// Update with discovered package information
    pub fn with_installed_info(
        mut self,
        provides_name: String,
        actual_package_name: Option<String>,
    ) -> Self {
        self.provides_name = provides_name;
        self.actual_package_name = actual_package_name;
        self
    }

    /// Get the state key for storage
    pub fn state_key(&self) -> String {
        format!("{}:{}", self.backend, self.provides_name)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            meta: StateMeta {
                schema_version: 3,
                last_sync: Utc::now(),
                hostname: "unknown".to_string(),
                last_update: None, // Default is never updated via declarch
                state_revision: Some(1),
                generator: Some("declarch".to_string()),
                features: None,
            },
            packages: HashMap::new(),
        }
    }
}
