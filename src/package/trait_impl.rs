use crate::utils::errors::Result;
use std::fmt;

pub trait PackageManager: Send + Sync {
    fn install(&self, packages: &[String]) -> Result<()>;
    fn remove(&self, packages: &[String]) -> Result<()>; 
    
    fn check(&self, package: &str) -> Result<bool>;
    fn get_installed(&self) -> Result<Vec<InstalledPackage>>;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

impl fmt::Display for InstalledPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.version)
    }
}
