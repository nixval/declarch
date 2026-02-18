mod cache_policy;
mod fetcher;
mod hint;
mod owner_detection;
mod types;
mod versioning;

use std::time::Duration;

pub use types::{InstallOwner, UpdateHint};
pub use versioning::{compare_versions, current_version};

const CACHE_TTL_SECS: i64 = 24 * 60 * 60;
const HTTP_TIMEOUT_SECS: u64 = 3;

pub fn is_managed_by_package_manager(owner: &InstallOwner) -> bool {
    hint::is_managed_by_package_manager(owner)
}

pub fn detect_install_owner() -> InstallOwner {
    owner_detection::detect_install_owner()
}

pub fn update_hint_cached() -> Option<UpdateHint> {
    hint::update_hint_cached(CACHE_TTL_SECS, Duration::from_secs(HTTP_TIMEOUT_SECS))
}

pub fn latest_version_live() -> Option<String> {
    fetcher::fetch_latest_version(Duration::from_secs(HTTP_TIMEOUT_SECS))
}

#[cfg(test)]
mod tests;
