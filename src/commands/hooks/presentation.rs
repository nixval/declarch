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
        println!("{}", render_hook_line(hook));
    }
}

pub(super) fn render_hook_line(hook: &LifecycleAction) -> String {
    let sudo_marker = matches!(hook.action_type, ActionType::Root);
    let package_info = if let Some(pkg) = &hook.package {
        format!(" [{}]", pkg.cyan())
    } else {
        String::new()
    };
    let safe_display = sanitize::sanitize_for_display(&hook.command);
    format!(
        "  {} {}{}",
        if sudo_marker { "🔒" } else { "→" },
        safe_display.cyan(),
        package_info
    )
}

#[cfg(test)]
mod tests {
    use super::render_hook_line;
    use crate::config::kdl::{ActionType, ErrorBehavior, LifecycleAction, LifecyclePhase};

    #[test]
    fn render_hook_line_uses_root_marker_and_package_tag() {
        let hook = LifecycleAction {
            command: "echo secure".to_string(),
            action_type: ActionType::Root,
            phase: LifecyclePhase::PreSync,
            package: Some("hyprland".to_string()),
            conditions: vec![],
            error_behavior: ErrorBehavior::Warn,
        };

        let line = render_hook_line(&hook);
        assert!(line.contains("🔒"));
        assert!(line.contains("echo secure"));
        assert!(line.contains("hyprland"));
    }

    #[test]
    fn render_hook_line_truncates_unsafe_long_display() {
        let hook = LifecycleAction {
            command: "x".repeat(220),
            action_type: ActionType::User,
            phase: LifecyclePhase::PreSync,
            package: None,
            conditions: vec![],
            error_behavior: ErrorBehavior::Warn,
        };

        let line = render_hook_line(&hook);
        assert!(line.contains("→"));
        assert!(line.contains("..."));
    }
}
