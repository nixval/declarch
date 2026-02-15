use crate::constants::{
    BACKENDS_FILE_NAME, CONFIG_EXTENSION, CONFIG_FILE_NAME, DECLARCH_DIR_NAME, MODULES_DIR_NAME,
    PROJECT_ORG, PROJECT_QUALIFIER, STATE_FILE_NAME,
};
use crate::error::{DeclarchError, Result};
use directories::{ProjectDirs, UserDirs};
use std::path::{Path, PathBuf};

pub fn expand_home(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();

    if !path_str.starts_with("~") {
        return Ok(path.to_path_buf());
    }

    let user_dirs = UserDirs::new().ok_or_else(|| {
        DeclarchError::PathError("Could not determine user home directory".to_string())
    })?;

    let home = user_dirs.home_dir();

    if path_str == "~" {
        return Ok(home.to_path_buf());
    }

    let stripped = path_str
        .strip_prefix("~/")
        .ok_or_else(|| DeclarchError::PathError(format!("Invalid path format: {}", path_str)))?;

    Ok(home.join(stripped))
}

pub fn config_dir() -> Result<PathBuf> {
    let proj = project_dirs()?;
    Ok(proj.config_dir().to_path_buf())
}

pub fn state_dir() -> Result<PathBuf> {
    let proj = project_dirs()?;
    let state_dir = proj.state_dir().ok_or_else(|| {
        DeclarchError::PathError("System does not support state directory".to_string())
    })?;
    Ok(state_dir.to_path_buf())
}

pub fn state_file() -> Result<PathBuf> {
    Ok(state_dir()?.join(STATE_FILE_NAME))
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE_NAME))
}

pub fn hosts_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("hosts"))
}

pub fn modules_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join(MODULES_DIR_NAME))
}

pub fn host_file(name: &str) -> Result<PathBuf> {
    Ok(hosts_dir()?.join(format!("{}.{}", name, CONFIG_EXTENSION)))
}

pub fn module_file(name: &str) -> Result<PathBuf> {
    Ok(modules_dir()?.join(format!("{}.{}", name, CONFIG_EXTENSION)))
}

pub fn backend_config() -> Result<PathBuf> {
    Ok(config_dir()?.join(BACKENDS_FILE_NAME))
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(PROJECT_QUALIFIER, PROJECT_ORG, DECLARCH_DIR_NAME).ok_or_else(|| {
        DeclarchError::PathError("Could not determine project directories".to_string())
    })
}
