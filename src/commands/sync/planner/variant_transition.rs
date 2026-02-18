use crate::config::loader;
use crate::core::{resolver, types::SyncTarget};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::state::types::State;
use crate::ui as output;
use colored::Colorize;

use super::InstalledSnapshot;

type VariantMismatch = (String, String, crate::core::types::Backend);

pub(super) fn collect_variant_mismatches(
    config: &loader::MergedConfig,
    installed_snapshot: &InstalledSnapshot,
    state: &State,
    tx: &resolver::Transaction,
    sync_target: &SyncTarget,
) -> Vec<VariantMismatch> {
    use crate::core::matcher::PackageMatcher;
    use crate::core::types::Backend;
    use std::collections::HashSet;

    let matcher = PackageMatcher::new();
    let mut variant_mismatches: Vec<VariantMismatch> = Vec::new();

    if matches!(sync_target, SyncTarget::All | SyncTarget::Backend(_)) {
        let available_backends: HashSet<Backend> = installed_snapshot
            .keys()
            .map(|pkg_id| pkg_id.backend.clone())
            .collect();

        for pkg_id in config
            .packages
            .keys()
            .filter(|pkg_id| available_backends.contains(&pkg_id.backend))
        {
            if tx.to_install.iter().any(|p| p.name == pkg_id.name) {
                continue;
            }

            if let Some(matched_id) = matcher.find_package(pkg_id, installed_snapshot)
                && matched_id.name != pkg_id.name
            {
                let state_key = resolver::make_state_key(pkg_id);
                let state_pkg = state.packages.get(&state_key);

                if state_pkg.is_none()
                    || state_pkg
                        .and_then(|s| s.actual_package_name.as_ref())
                        .map(|n| n != &matched_id.name)
                        .unwrap_or(false)
                {
                    variant_mismatches.push((
                        pkg_id.name.clone(),
                        matched_id.name,
                        pkg_id.backend.clone(),
                    ));
                }
            }
        }
    }

    variant_mismatches
}

pub(super) fn emit_variant_transition_error(variant_mismatches: &[VariantMismatch]) -> Result<()> {
    output::separator();
    output::error("Variant transition detected!");
    println!("\nThe following packages have different variants installed:\n");

    for (config_name, installed_name, backend) in variant_mismatches {
        println!(
            "  [{}] {}  →  {}",
            backend.to_string().dimmed(),
            config_name.cyan().bold(),
            installed_name.yellow().bold()
        );
    }

    println!(
        "\n{}",
        "This requires explicit transition to avoid unintended changes.".dimmed()
    );
    println!("\n{}", "To resolve this:".bold());
    println!("  1. For each package, run:");
    for (config_name, installed_name, backend) in variant_mismatches {
        let backend_prefix = format!("{}:", backend);
        println!(
            "     {}",
            project_identity::cli_with(&format!(
                "switch {}{} {}{}",
                backend_prefix, installed_name, backend_prefix, config_name
            ))
            .bold()
        );
    }
    println!("\n  2. Or, update your config to match the installed variant:");
    for (config_name, installed_name, backend) in variant_mismatches {
        println!(
            "       pkg {{ {} {{ {} }} }}  (was: {})",
            backend.to_string().cyan(),
            installed_name.yellow(),
            config_name.dimmed()
        );
    }
    println!(
        "\n  3. Use {} to bypass this check (not recommended)",
        "--force".yellow().bold()
    );

    Err(DeclarchError::Other(format!(
        "Variant transition required. Use '{}' or update your config.",
        project_identity::cli_with("switch")
    )))
}
