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

    /// Actual AUR package name (if different from provides_name)
    /// None for flatpak or when aur_package_name == provides_name
    #[serde(default)]
    pub aur_package_name: Option<String>,

    pub installed_at: DateTime<Utc>,
    pub version: Option<String>,
}

// Implement Default for PackageState to help with serde defaults
impl Default for PackageState {
    fn default() -> Self {
        Self {
            backend: Backend::Aur,
            config_name: String::new(),
            provides_name: String::new(),
            aur_package_name: None,
            installed_at: Utc::now(),
            version: None,
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
            aur_package_name: None,
            installed_at: Utc::now(),
            version,
        }
    }

    /// Update with discovered package information
    pub fn with_installed_info(
        mut self,
        provides_name: String,
        aur_package_name: Option<String>,
    ) -> Self {
        self.provides_name = provides_name;
        self.aur_package_name = aur_package_name;
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
                schema_version: 2,
                last_sync: Utc::now(),
                hostname: "unknown".to_string(),
                last_update: None, // Default is never updated via declarch
            },
            packages: HashMap::new(),
        }
    }
}
