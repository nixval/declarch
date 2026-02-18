use crate::backends::config::BackendConfig;
use crate::error::{DeclarchError, Result};
use kdl::KdlNode;

pub(super) fn parse_install_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.install_cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Install command required. Usage: install \"command\"".to_string())
        })?
        .to_string();

    Ok(())
}

pub(super) fn parse_remove_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Remove command required. Usage: remove \"command\"".to_string())
        })?
        .to_string();

    if cmd != "-" {
        config.remove_cmd = Some(cmd);
    }
    Ok(())
}

pub(super) fn parse_update_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Update command required. Usage: update \"command\"".to_string())
        })?
        .to_string();

    if cmd != "-" {
        config.update_cmd = Some(cmd);
    }
    Ok(())
}

pub(super) fn parse_cache_clean_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "Cache clean command required. Usage: cache_clean \"command\"".to_string(),
            )
        })?
        .to_string();

    if cmd != "-" {
        config.cache_clean_cmd = Some(cmd);
    }
    Ok(())
}

pub(super) fn parse_upgrade_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other("Upgrade command required. Usage: upgrade \"command\"".to_string())
        })?
        .to_string();

    if cmd != "-" {
        config.upgrade_cmd = Some(cmd);
    }
    Ok(())
}

pub(super) fn parse_noconfirm(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.noconfirm_flag = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .map(|s| s.to_string());
    Ok(())
}

pub(super) fn parse_fallback(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.fallback = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .map(|s| s.to_string());
    Ok(())
}
