use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Aur,     
    Flatpak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageState {
    pub backend: Backend,
    pub installed_at: DateTime<Utc>,
    pub version: Option<String>, 
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
