use crate::constants::CRITICAL_PACKAGES;
use crate::core::resolver;
use crate::core::types::PackageId;
use crate::project_identity;
use crate::ui as output;
use colored::Colorize;
use std::collections::HashMap;

use super::InstalledSnapshot;

pub(super) fn display_transaction_plan_impl(tx: &resolver::Transaction, should_prune: bool) {
    let has_changes = !tx.to_install.is_empty()
        || !tx.to_adopt.is_empty()
        || (!tx.to_prune.is_empty() && should_prune);

    if !has_changes {
        return;
    }

    println!("{}", "Changes:".green().bold());

    if !tx.to_install.is_empty() {
        let groups = group_by_backend(&tx.to_install);
        println!(
            "  {} {}",
            "Install:".green(),
            format_backend_groups(&groups)
        );
    }

    if !tx.to_adopt.is_empty() {
        let groups = group_by_backend(&tx.to_adopt);
        println!(
            "  {}   {}",
            "Adopt:".yellow(),
            format_backend_groups(&groups)
        );
    }

    if !tx.to_prune.is_empty() && should_prune {
        let groups = group_by_backend(&tx.to_prune);
        let formatted: HashMap<String, Vec<String>> = groups
            .into_iter()
            .map(|(backend, packages)| {
                let modified: Vec<String> = packages
                    .into_iter()
                    .map(|p| {
                        if CRITICAL_PACKAGES.contains(&p.as_str()) {
                            format!("{} [keep]", p)
                        } else {
                            p
                        }
                    })
                    .collect();
                (backend, modified)
            })
            .collect();
        println!(
            "  {}  {}",
            "Remove:".red(),
            format_backend_groups(&formatted)
        );
    }
}

pub(super) fn display_dry_run_details_impl(
    tx: &resolver::Transaction,
    should_prune: bool,
    installed_snapshot: &InstalledSnapshot,
) {
    let has_changes = !tx.to_install.is_empty()
        || !tx.to_adopt.is_empty()
        || (!tx.to_prune.is_empty() && should_prune);

    if !has_changes {
        output::success("Dry-run: No changes needed - everything is up to date!");
        return;
    }

    output::separator();
    println!("{}", "🧪 DRY-RUN SIMULATION".cyan().bold());
    println!(
        "{}",
        "   No changes will be made to your system.\n".dimmed()
    );

    let install_count = tx.to_install.len();
    let adopt_count = tx.to_adopt.len();
    let prune_count = if should_prune { tx.to_prune.len() } else { 0 };

    println!("{}", "Summary:".bold());
    if install_count > 0 {
        println!(
            "  • {} new package(s) to install",
            install_count.to_string().green()
        );
    }
    if adopt_count > 0 {
        println!(
            "  • {} package(s) to adopt into state",
            adopt_count.to_string().yellow()
        );
    }
    if prune_count > 0 {
        println!("  • {} package(s) to remove", prune_count.to_string().red());
    }
    println!();

    if !tx.to_install.is_empty() {
        println!("{}", "Packages to install:".green().bold());
        display_package_groups_detailed(&tx.to_install, installed_snapshot);
    }

    if !tx.to_adopt.is_empty() {
        println!("{}", "\nPackages to adopt:".yellow().bold());
        display_package_groups_detailed(&tx.to_adopt, installed_snapshot);
    }

    if !tx.to_prune.is_empty() && should_prune {
        println!("{}", "\nPackages to remove:".red().bold());
        let groups = group_by_backend(&tx.to_prune);
        for (backend, packages) in groups {
            println!("  {}:", backend.cyan());
            for pkg in packages {
                if CRITICAL_PACKAGES.contains(&pkg.as_str()) {
                    println!(
                        "    • {} {} (protected)",
                        pkg.red(),
                        "[will be kept]".yellow()
                    );
                } else {
                    println!("    • {}", pkg.red());
                }
            }
        }
    }

    println!();
    output::separator();
    println!("{}", "Pre-flight Checks:".bold());

    let already_installed: Vec<_> = tx
        .to_install
        .iter()
        .filter(|pkg| {
            installed_snapshot
                .keys()
                .any(|pkg_id| pkg_id.name == pkg.name)
        })
        .collect();

    if !already_installed.is_empty() {
        output::warning(&format!(
            "{} package(s) appear to already be installed but not tracked in state",
            already_installed.len()
        ));
        println!(
            "  They will be 'adopted' into {} state management.",
            project_identity::BINARY_NAME
        );
    } else {
        println!("  ✅ No conflicts detected");
    }

    println!();
    output::info("To apply these changes, run without --dry-run flag");
}

fn group_by_backend(packages: &[PackageId]) -> HashMap<String, Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in packages {
        groups
            .entry(pkg.backend.to_string())
            .or_default()
            .push(pkg.name.clone());
    }
    for packages in groups.values_mut() {
        packages.sort();
    }
    groups
}

fn format_backend_groups(groups: &HashMap<String, Vec<String>>) -> String {
    let mut backends: Vec<_> = groups.keys().collect();
    backends.sort();

    let mut result = String::new();
    for (i, backend) in backends.iter().enumerate() {
        if let Some(packages) = groups.get(*backend) {
            if i > 0 {
                result.push_str("\n         ");
            }
            result.push_str(&format!(
                "({}): {}",
                backend.cyan(),
                packages.join(", ").dimmed()
            ));
        }
    }
    result
}

fn display_package_groups_detailed(packages: &[PackageId], installed_snapshot: &InstalledSnapshot) {
    let groups = group_by_backend(packages);
    for (backend, pkg_names) in groups {
        println!("  {}:", backend.cyan());
        for name in pkg_names {
            let variant_info = installed_snapshot
                .iter()
                .find(|(pkg_id, _)| pkg_id.name == name && pkg_id.backend.to_string() == backend);

            if let Some((_, meta)) = variant_info {
                if let Some(ref version) = meta.version {
                    println!(
                        "    • {} {}",
                        name.green(),
                        format!("(v{} already installed)", version).dimmed()
                    );
                } else {
                    println!(
                        "    • {} {}",
                        name.green(),
                        "(already installed, untracked)".dimmed()
                    );
                }
            } else {
                println!("    • {}", name.green());
            }
        }
    }
}
