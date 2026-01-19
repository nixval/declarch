use crate::error::Result;
use crate::core::types::{Backend, PackageMetadata}; // PackageId dihapus
use std::collections::HashMap;

pub trait PackageManager: Send + Sync {
    fn backend_type(&self) -> Backend;
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>>;
    fn install(&self, packages: &[String]) -> Result<()>;
    fn remove(&self, packages: &[String]) -> Result<()>;
    fn is_available(&self) -> bool;

    /// Get list of packages that depend on the given package (reverse dependencies)
    fn get_required_by(&self, package: &str) -> Result<Vec<String>>;
}
