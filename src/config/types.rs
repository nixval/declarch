use serde::{Deserialize, Serialize};

/// Global declarch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Which AUR helper to use (paru or yay)
    #[serde(default = "default_aur_helper")]
    pub aur_helper: AurHelper,

    /// Default editor for editing configs
    #[serde(default = "default_editor")]
    pub editor: String,

    /// Other settings (future)
    #[serde(default)]
    pub other: std::collections::HashMap<String, String>,
}

fn default_aur_helper() -> AurHelper {
    AurHelper::Paru
}

fn default_editor() -> String {
    // Try to detect preferred editor, fallback to nano
    if let Ok(ed) = std::env::var("EDITOR")
        && !ed.is_empty()
    {
        return ed;
    }
    if let Ok(ed) = std::env::var("VISUAL")
        && !ed.is_empty()
    {
        return ed;
    }
    "nano".to_string() // Default fallback
}

/// AUR helper choice
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AurHelper {
    Paru,
    Yay,
}

impl std::fmt::Display for AurHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Paru => write!(f, "paru"),
            Self::Yay => write!(f, "yay"),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            aur_helper: AurHelper::Paru,
            editor: default_editor(),
            other: std::collections::HashMap::new(),
        }
    }
}

/// Host-specific configuration
#[derive(Debug, Clone, Default)]
pub struct HostConfig {
    pub description: Option<String>,
    pub modules: Vec<String>,
    pub packages: Vec<String>,
    pub exclude: Vec<String>,
    pub conflicts: Vec<String>,
}

/// Module configuration
#[derive(Debug, Clone, Default)]
pub struct ModuleConfig {
    pub description: Option<String>,
    pub packages: Vec<String>,
}

#[cfg(test)]
mod tests;
