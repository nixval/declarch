use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;
use chrono::{DateTime, Utc};

// Core identifier for any package managed by declarch
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId {
    pub name: String,
    pub backend: Backend,
}

// Supported backends.
// To add a new package manager (e.g. Snap), add a variant here and update:
// - Backend::display()
// - Backend::from_str() (via PackageId::from_str())
// - BackendRegistry::register_defaults()
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Aur,      // Handles Pacman & AUR (Arch Linux)
    Flatpak,  // Flatpak (Cross-distro)
    Soar,     // Soar (Cross-distro static binaries)
    // Future: Snap, Cargo, Nix, Nala, etc.
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aur => write!(f, "aur"),
            Self::Flatpak => write!(f, "flatpak"),
            Self::Soar => write!(f, "soar"),
        }
    }
}
impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.backend {
            Backend::Aur => write!(f, "{}", self.name),
            Backend::Flatpak => write!(f, "flatpak:{}", self.name),
            Backend::Soar => write!(f, "{}", self.name), // Soar packages displayed without prefix
        }
    }
}
// Parsing logic centralized here.
// Handles "flatpak:name" vs "soar:name" vs "name"
impl FromStr for PackageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(name) = s.strip_prefix("flatpak:") {
            Ok(PackageId {
                name: name.to_string(),
                backend: Backend::Flatpak,
            })
        } else if let Some(name) = s.strip_prefix("soar:") {
            // Explicit 'soar:' prefix support
            Ok(PackageId {
                name: name.to_string(),
                backend: Backend::Soar,
            })
        } else if let Some(name) = s.strip_prefix("aur:") {
            // Explicit 'aur:' prefix support (optional but explicit)
            Ok(PackageId {
                name: name.to_string(),
                backend: Backend::Aur,
            })
        } else {
            // Default to AUR/Native if no prefix provided
            Ok(PackageId {
                name: s.to_string(),
                backend: Backend::Aur,
            })
        }
    }
}

// Rich metadata for state tracking.
// Captures version to detect drift in future updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub version: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub source_file: Option<String>, // Tracks which .kdl file defined this
}

// Handles the "Targeting" logic for partial syncs.
// Resolves ambiguity between Package names and Module paths.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncTarget {
    All,
    Backend(Backend),
    // User provided a specific string (e.g., "git" or "modules/gaming")
    // The resolver will check modules first, then packages.
    Named(String), 
}
