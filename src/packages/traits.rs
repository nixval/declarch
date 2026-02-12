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

    /// Update package list/index (e.g., apt update, pacman -Sy)
    /// Default: no update support
    fn update(&self) -> Result<()> {
        Ok(())
    }

    /// Check if this package manager supports update
    fn supports_update(&self) -> bool {
        false // Default: no update support
    }

    /// Clean package cache
    /// Default: no cache clean support
    fn clean_cache(&self) -> Result<()> {
        Ok(())
    }

    /// Check if this package manager supports cache cleaning
    fn supports_cache_clean(&self) -> bool {
        false // Default: no cache clean support
    }
}
