use crate::core::types::{Backend, PackageMetadata}; // PackageId dihapus
use crate::error::Result;
use std::collections::HashMap;

/// Search result from package repository
#[derive(Debug, Clone)]
pub struct PackageSearchResult {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub backend: Backend,
}

pub trait PackageManager: Send + Sync {
    fn backend_type(&self) -> Backend;
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>>;
    fn install(&self, packages: &[String]) -> Result<()>;
    fn remove(&self, packages: &[String]) -> Result<()>;
    fn is_available(&self) -> bool;

    /// Get list of packages that depend on the given package (reverse dependencies)
    fn get_required_by(&self, package: &str) -> Result<Vec<String>>;

    /// Search for packages in the repository
    fn search(&self, _query: &str) -> Result<Vec<PackageSearchResult>> {
        Ok(Vec::new()) // Default: no search support
    }

    /// Check if this package manager supports search
    fn supports_search(&self) -> bool {
        false // Default: no search support
    }
}
