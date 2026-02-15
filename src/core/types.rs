use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Core identifier for any package managed by declarch
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId {
    pub name: String,
    pub backend: Backend,
}

/// Backend identifier - purely generic, no hardcoded variants
///
/// In v0.6+, all backends are treated equally. There are no special cases
/// for system package managers, npm, etc. Each backend is defined by its configuration in
/// ~/.config/declarch/backends/*.kdl
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Backend(pub String);

impl Backend {
    /// Create a new backend from a string
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into().to_lowercase())
    }

    /// Get the backend name
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Normalize to lowercase
        Ok(Self(s.to_lowercase()))
    }
}

impl From<&str> for Backend {
    fn from(s: &str) -> Self {
        Self(s.to_lowercase())
    }
}

impl From<String> for Backend {
    fn from(s: String) -> Self {
        Self(s.to_lowercase())
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Always display with backend prefix for clarity
        write!(f, "{}:{}", self.backend.0, self.name)
    }
}

/// Parse package string with optional backend prefix
///
/// Supports formats:
/// - "package" -> uses default backend (first in config or explicit default)
/// - "backend:package" -> specific backend
///
/// Note: In v0.6+, there's no implicit default. The default backend
/// should be specified in declarch.kdl
impl FromStr for PackageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for backend prefix format "backend:package"
        if let Some(colon_pos) = s.find(':') {
            let backend_name = &s[..colon_pos];
            let package_name = &s[colon_pos + 1..];

            // Validate backend name (alphanumeric, hyphens, underscores)
            if backend_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && !backend_name.is_empty()
                && !package_name.is_empty()
            {
                return Ok(PackageId {
                    name: package_name.to_string(),
                    backend: Backend::from(backend_name),
                });
            }
        }

        // No valid prefix found - this will be handled by the caller
        // who should provide a default backend context
        Err(format!(
            "Invalid package format '{}'. Use 'backend:package' format.",
            s
        ))
    }
}

/// Parse with explicit default backend
impl PackageId {
    /// Parse package string with a default backend
    ///
    /// If no backend prefix is provided, uses the given default.
    pub fn parse_with_default(s: &str, default_backend: &Backend) -> Result<Self, String> {
        // Try to parse with prefix first
        if let Some(colon_pos) = s.find(':') {
            let backend_name = &s[..colon_pos];
            let package_name = &s[colon_pos + 1..];

            // Validate backend name
            if backend_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && !backend_name.is_empty()
                && !package_name.is_empty()
            {
                return Ok(PackageId {
                    name: package_name.to_string(),
                    backend: Backend::from(backend_name),
                });
            }
        }

        // No valid prefix, use default
        Ok(PackageId {
            name: s.to_string(),
            backend: default_backend.clone(),
        })
    }
}

/// Rich metadata for state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub version: Option<String>,
    pub variant: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub source_file: Option<String>,
}

/// Sync target for partial syncs
#[derive(Debug, Clone, PartialEq)]
pub enum SyncTarget {
    All,
    Backend(Backend),
    Named(String),
}
