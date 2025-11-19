use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub meta: StateMeta,
    pub packages: HashMap<String, PackageState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMeta {
    pub schema_version: u8,
    pub last_sync: DateTime<Utc>,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Aur,     // Handles both repo & AUR via paru/yay
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
                schema_version: 1,
                last_sync: Utc::now(),
                hostname: "unknown".to_string(),
            },
            packages: HashMap::new(),
        }
    }
}
