use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

pub mod aur;
pub mod generic;

pub use aur::AurParser;
pub use generic::GenericBackendParser;

/// Trait for backend-specific package parsing
///
/// Each backend (AUR, Soar, Flatpak) implements this trait
/// to define how it parses packages from KDL nodes.
pub trait BackendParser: Send + Sync {
    /// Backend identifier (e.g., "aur", "soar", "flatpak")
    fn name(&self) -> &'static str;

    /// Aliases for this backend (e.g., "app" is an alias for "soar")
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// Parse packages from a KDL node and add them to the config
    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()>;

    /// Check if a backend name matches this parser (including aliases)
    fn matches(&self, backend: &str) -> bool {
        self.name() == backend || self.aliases().contains(&backend)
    }
}
