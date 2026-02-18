use crate::project_identity;
use crate::ui as output;
use crate::utils::update_check::{InstallOwner, is_managed_by_package_manager, update_hint_cached};

pub(crate) fn maybe_print_update_notification() {
    let Some(hint) = update_hint_cached() else {
        return;
    };

    output::separator();
    output::warning(&format!(
        "New {} release available: {} -> {}",
        project_identity::BINARY_NAME,
        hint.current,
        hint.latest
    ));

    if is_managed_by_package_manager(&hint.owner) {
        let msg = match hint.owner {
            InstallOwner::Pacman => format!(
                "Update using package manager: paru -Syu {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Homebrew => format!(
                "Update using package manager: brew upgrade {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Scoop => format!(
                "Update using package manager: scoop update {}",
                project_identity::BINARY_NAME
            ),
            InstallOwner::Winget => format!(
                "Update using package manager: winget upgrade {}",
                project_identity::BINARY_NAME
            ),
            _ => "Update using your package manager".to_string(),
        };
        output::info(&msg);
    } else {
        output::info(&format!(
            "For script/manual install, run: {} self-update",
            project_identity::BINARY_NAME
        ));
    }
}
