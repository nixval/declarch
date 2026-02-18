use crate::utils::update_check::cache_policy;
use crate::utils::update_check::owner_detection;
use crate::utils::update_check::types::{InstallOwner, UpdateHint};
use crate::utils::update_check::versioning;
use std::time::Duration;

pub(super) fn is_managed_by_package_manager(owner: &InstallOwner) -> bool {
    matches!(
        owner,
        InstallOwner::Pacman | InstallOwner::Homebrew | InstallOwner::Scoop | InstallOwner::Winget
    )
}

pub(super) fn update_hint_cached(cache_ttl_secs: i64, timeout: Duration) -> Option<UpdateHint> {
    let current = versioning::current_version();
    let latest = cache_policy::get_latest_version_cached(cache_ttl_secs, timeout)?;
    if versioning::compare_versions(&latest, &current).is_gt() {
        Some(UpdateHint {
            current,
            latest,
            owner: owner_detection::detect_install_owner(),
        })
    } else {
        None
    }
}
