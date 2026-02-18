use crate::config::loader::MergedConfig;
use crate::error::Result;
use crate::ui as output;
use std::collections::HashSet;

pub(super) fn show_diff(config: &MergedConfig) -> Result<()> {
    use crate::core::types::PackageId;

    let state_path = crate::state::io::get_state_path()?;
    if !state_path.exists() {
        output::info("Packages to install:");
        for pkg_id in config.packages.keys() {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
        return Ok(());
    }

    let state = crate::state::io::load_state()?;
    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
    let mut state_set: HashSet<PackageId> = HashSet::new();

    for pkg_state in state.packages.values() {
        state_set.insert(PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        });
    }

    let to_install: Vec<_> = config_set.difference(&state_set).cloned().collect();
    let to_remove: Vec<_> = state_set.difference(&config_set).cloned().collect();

    if to_install.is_empty() && to_remove.is_empty() {
        output::info("No changes planned");
        return Ok(());
    }

    if !to_install.is_empty() {
        output::info("To install:");
        for pkg_id in &to_install {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    if !to_remove.is_empty() {
        output::info("To remove:");
        for pkg_id in &to_remove {
            println!("  - {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    Ok(())
}
