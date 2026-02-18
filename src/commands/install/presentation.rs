use crate::ui as output;

use super::InstallOptions;
use super::planning::PlannedInstall;

pub(super) fn show_dry_run_install(options: &InstallOptions, planned_installs: &[PlannedInstall]) {
    output::header("Dry Run: Install");
    output::keyval("Packages", &planned_installs.len().to_string());
    output::keyval(
        "Target module",
        options
            .module
            .as_deref()
            .unwrap_or("modules/others.kdl (default)"),
    );
    output::keyval(
        "Auto sync",
        if options.no_sync {
            "disabled"
        } else {
            "enabled"
        },
    );

    for planned in planned_installs {
        output::indent(&format!("+ {}:{}", planned.backend, planned.package), 1);
    }

    if options.verbose {
        output::separator();
        output::info("Resolution details:");
        for raw in &options.packages {
            output::indent(&format!("input: {}", raw), 1);
        }
    }
}

pub(super) fn show_install_summary(verbose: bool, added_packages: &[String]) {
    if verbose && !added_packages.is_empty() {
        output::verbose(&format!("Packages added: {}", added_packages.join(", ")));
    }
}
