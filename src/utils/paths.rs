use std::path::{Path, PathBuf};
use crate::error::{DeclarchError, Result};
use directories::{UserDirs, ProjectDirs};

pub fn expand_home(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();

    if !path_str.starts_with("~") {
        return Ok(path.to_path_buf());
    }

    let user_dirs = UserDirs::new()
        .ok_or_else(|| DeclarchError::Other("Could not determine user home directory".to_string()))?;
    
    let home = user_dirs.home_dir();
    
    if path_str == "~" {
        return Ok(home.to_path_buf());
    }

    let stripped = path_str.strip_prefix("~/")
        .ok_or_else(|| DeclarchError::Other(format!("Invalid path format: {}", path_str)))?;

    Ok(home.join(stripped))
}

pub fn config_dir() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "declarch", "declarch")
        .ok_or_else(|| DeclarchError::Other("Could not determine config directory".to_string()))?;
    Ok(proj.config_dir().to_path_buf())
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("declarch.kdl"))
}

pub fn hosts_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("hosts"))
}

pub fn modules_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("modules"))
}

pub fn host_file(name: &str) -> Result<PathBuf> {
    Ok(hosts_dir()?.join(format!("{}.kdl", name)))
}

pub fn module_file(name: &str) -> Result<PathBuf> {
    Ok(modules_dir()?.join(format!("{}.kdl", name)))
}
