use crate::config::kdl::{ActionType, LifecycleAction};
use crate::project_identity;
use crate::ui as output;
use crate::utils::sanitize;
use colored::Colorize;

pub(super) fn show_disabled_hooks_warning(hooks: &[&LifecycleAction], phase_name: &str) {
    output::warning(&format!(
        "{} hooks detected but not executed (either --hooks not provided or blocked by policy)",
        phase_name
    ));
    display_hooks(hooks, phase_name, true);

    println!("\n{}", "⚠️  Security Warning:".yellow().bold());
    println!(
        "{}",
        "   Hooks can execute arbitrary system commands.".yellow()
    );
    println!(
        "{}",
        "   Only enable hooks from sources you trust.".yellow()
    );

    println!("\n{}", "To enable hooks after reviewing:".dimmed());
    println!("  {}", project_identity::cli_with("sync --hooks").bold());
    println!("  {}", "dc sync --hooks".dimmed());

    println!("\n{}", "To review the full config:".dimmed());
    println!(
        "  {}",
        format!(
            "cat ~/.config/{}/{}",
            project_identity::CONFIG_DIR_NAME,
            project_identity::CONFIG_FILE_BASENAME
        )
        .dimmed()
    );
}

pub(super) fn display_hooks(hooks: &[&LifecycleAction], title: &str, warn_mode: bool) {
    if warn_mode {
        println!("\n{}:", title.yellow().bold());
    } else {
        println!("\n{}:", title.cyan().bold());
    }

    for hook in hooks {
        let sudo_marker = matches!(hook.action_type, ActionType::Root);
        let package_info = if let Some(pkg) = &hook.package {
            format!(" [{}]", pkg.cyan())
        } else {
            String::new()
        };
        let safe_display = sanitize::sanitize_for_display(&hook.command);
        println!(
            "  {} {}{}",
            if sudo_marker { "🔒" } else { "→" },
            safe_display.cyan(),
            package_info
        );
    }
}
