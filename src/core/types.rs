use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// Core identifier for any package managed by declarch
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId {
    pub name: String,
    pub backend: Backend,
}

// Supported backends.
// To add a new package manager (e.g. Snap), add a variant here and update:
// - Backend::display() (fmt::Display trait)
// - Backend::from_str() (FromStr trait)
// - PackageId::from_str() for prefix parsing
// - BackendRegistry::register_defaults()
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Aur,            // Handles Pacman & AUR (Arch Linux)
    Flatpak,        // Flatpak (Cross-distro)
    Soar,           // Soar (Cross-distro static binaries)
    Npm,            // npm (Node.js global packages)
    Yarn,           // Yarn global packages
    Pnpm,           // pnpm global packages
    Bun,            // Bun global packages
    Pip,            // pip (Python packages)
    Cargo,          // Cargo (Rust packages)
    Brew,           // Homebrew (macOS/Linux)
    Custom(String), // User-defined backends (nala, zypper, dnf5, etc.)
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aur => write!(f, "aur"),
            Self::Flatpak => write!(f, "flatpak"),
            Self::Soar => write!(f, "soar"),
            Self::Npm => write!(f, "npm"),
            Self::Yarn => write!(f, "yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Bun => write!(f, "bun"),
            Self::Pip => write!(f, "pip"),
            Self::Cargo => write!(f, "cargo"),
            Self::Brew => write!(f, "brew"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aur" => Ok(Self::Aur),
            "flatpak" => Ok(Self::Flatpak),
            "soar" => Ok(Self::Soar),
            "npm" => Ok(Self::Npm),
            "yarn" => Ok(Self::Yarn),
            "pnpm" => Ok(Self::Pnpm),
            "bun" => Ok(Self::Bun),
            "pip" => Ok(Self::Pip),
            "cargo" => Ok(Self::Cargo),
            "brew" => Ok(Self::Brew),
            // Treat unknown backends as Custom
            other => Ok(Self::Custom(other.to_string())),
        }
    }
}
impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.backend {
            Backend::Aur => write!(f, "{}", self.name),
            Backend::Flatpak => write!(f, "flatpak:{}", self.name),
            Backend::Soar => write!(f, "{}", self.name), // Soar packages displayed without prefix
            Backend::Npm => write!(f, "npm:{}", self.name),
            Backend::Yarn => write!(f, "yarn:{}", self.name),
            Backend::Pnpm => write!(f, "pnpm:{}", self.name),
            Backend::Bun => write!(f, "bun:{}", self.name),
            Backend::Pip => write!(f, "pip:{}", self.name),
            Backend::Cargo => write!(f, "cargo:{}", self.name),
            Backend::Brew => write!(f, "brew:{}", self.name),
            Backend::Custom(name) => write!(f, "{}:{}", name, self.name),
        }
    }
}
// Parsing logic centralized here.
// Handles "flatpak:name" vs "npm:name" vs "custom-backend:name" vs "name"
impl FromStr for PackageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for backend prefixes
        let prefixes = [
            ("flatpak:", Backend::Flatpak),
            ("soar:", Backend::Soar),
            ("npm:", Backend::Npm),
            ("yarn:", Backend::Yarn),
            ("pnpm:", Backend::Pnpm),
            ("bun:", Backend::Bun),
            ("pip:", Backend::Pip),
            ("cargo:", Backend::Cargo),
            ("brew:", Backend::Brew),
            ("aur:", Backend::Aur),
        ];

        for (prefix, backend) in prefixes {
            if let Some(name) = s.strip_prefix(prefix) {
                return Ok(PackageId {
                    name: name.to_string(),
                    backend,
                });
            }
        }

        // Check for custom backend format "backend-name:package"
        // This handles any unknown backend prefix (e.g., "nala:vim", "zypper:firefox")
        if let Some(colon_pos) = s.find(':') {
            let backend_name = &s[..colon_pos];
            let package_name = &s[colon_pos + 1..];

            // Only treat as custom backend if it looks like a valid backend name
            // (alphanumeric, hyphens, underscores)
            if backend_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Ok(PackageId {
                    name: package_name.to_string(),
                    backend: Backend::Custom(backend_name.to_string()),
                });
            }
        }

        // Default to AUR/Native if no prefix provided
        Ok(PackageId {
            name: s.to_string(),
            backend: Backend::Aur,
        })
    }
}

// Rich metadata for state tracking.
// Captures version to detect drift in future updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub version: Option<String>,
    pub variant: Option<String>, // Package variant (e.g., "git", "bin", or full Soar variant)
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
