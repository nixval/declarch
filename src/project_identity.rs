//! Central project identity contract.
//!
//! This module is the single source of truth for runtime identity values.
//! Keep `stable_project_id` stable across rename transitions.

pub const DISPLAY_NAME: &str = "Declarch";
pub const BINARY_NAME: &str = "declarch";
pub const BINARY_ALIASES: &[&str] = &["decl"];
pub const STABLE_PROJECT_ID: &str = "declarch";
pub const CONFIG_DIR_NAME: &str = "declarch";
pub const STATE_DIR_NAME: &str = "declarch";
pub const ENV_PREFIX: &str = "DECLARCH";
pub const RELEASE_ASSET_PREFIX: &str = "declarch";
pub const REPO_SLUG: &str = "nixval/declarch";
pub const REGISTRY_SLUG: &str = "nixval/declarch-packages";
pub const CONFIG_FILE_BASENAME: &str = "declarch.kdl";

pub fn env_key(suffix: &str) -> String {
    format!("{}_{}", ENV_PREFIX, suffix)
}

pub fn registry_raw_base_url() -> String {
    format!("https://raw.githubusercontent.com/{}/main", REGISTRY_SLUG)
}

pub fn release_download_base_url(version: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/v{}",
        REPO_SLUG, version
    )
}

pub fn github_latest_release_api() -> String {
    format!("https://api.github.com/repos/{}/releases/latest", REPO_SLUG)
}
