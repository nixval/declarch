use crate::error::Result;
use crate::core::types::{Backend, PackageMetadata}; // PackageId dihapus
use std::collections::HashMap;

pub trait PackageManager: Send + Sync {
    fn backend_type(&self) -> Backend;
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>>;
    fn install(&self, packages: &[String]) -> Result<()>;
    fn remove(&self, packages: &[String]) -> Result<()>;
    fn is_available(&self) -> bool;
}
