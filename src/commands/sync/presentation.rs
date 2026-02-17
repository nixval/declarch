use super::{InstalledSnapshot, SyncOptions, SyncPreviewReport};
use crate::core::types::{PackageId, SyncTarget};
use crate::project_identity;
use crate::ui as output;

pub(super) fn build_sync_preview_report(
    options: &SyncOptions,
    sync_target: &SyncTarget,
    transaction: &crate::core::resolver::Transaction,
) -> SyncPreviewReport {
    SyncPreviewReport {
        dry_run: true,
        prune: options.prune,
        update: options.update,
        target: sync_target_to_string(sync_target),
        install_count: transaction.to_install.len(),
        remove_count: transaction.to_prune.len(),
        adopt_count: transaction.to_adopt.len(),
        to_install: transaction
            .to_install
            .iter()
            .map(package_id_to_string)
            .collect(),
        to_remove: transaction
            .to_prune
            .iter()
            .map(package_id_to_string)
            .collect(),
        to_adopt: transaction
            .to_adopt
            .iter()
            .map(package_id_to_string)
            .collect(),
    }
}

fn package_id_to_string(pkg: &PackageId) -> String {
    format!("{}:{}", pkg.backend, pkg.name)
}

pub(super) fn sync_target_to_string(target: &SyncTarget) -> String {
    match target {
        SyncTarget::All => "all".to_string(),
        SyncTarget::Backend(b) => format!("backend:{}", b),
        SyncTarget::Named(name) => format!("named:{}", name),
    }
}

/// Show diff view of sync changes.
pub(super) fn show_sync_diff(
    transaction: &crate::core::resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
) {
    use colored::Colorize;

    output::header("Sync Diff");

    if !transaction.to_install.is_empty() {
        println!("\n{}:", "Packages to install".green().bold());
        for pkg_id in &transaction.to_install {
            println!("  {} {} {}", "+".green(), pkg_id.backend, pkg_id.name);
        }
    }

    if !transaction.to_prune.is_empty() {
        println!("\n{}:", "Packages to remove".red().bold());
        for pkg_id in &transaction.to_prune {
            let version = installed_snapshot
                .get(pkg_id)
                .and_then(|m| m.version.as_ref())
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            println!(
                "  {} {} {}{}",
                "-".red(),
                pkg_id.backend,
                pkg_id.name,
                version.dimmed()
            );
        }
    }

    if !transaction.to_adopt.is_empty() {
        println!("\n{}:", "Packages to adopt".yellow().bold());
        for pkg_id in &transaction.to_adopt {
            println!("  {} {} {}", "~".yellow(), pkg_id.backend, pkg_id.name);
        }
    }

    println!();
    let total_changes =
        transaction.to_install.len() + transaction.to_prune.len() + transaction.to_adopt.len();
    output::info(&format!("Total changes: {}", total_changes));
    output::info(&format!(
        "Run '{}' to apply these changes",
        project_identity::cli_with("sync")
    ));
}
