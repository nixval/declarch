use super::{InfoReasonOptions, MergedConfig, PackageId};
use crate::project_identity;
use crate::ui as output;
use std::path::Path;

pub(super) fn show_active_context(options: &InfoReasonOptions, config: &MergedConfig) {
    output::header("Active Context");

    output::keyval(
        "Profile",
        options.profile.as_deref().unwrap_or("(default only)"),
    );
    output::keyval("Host", options.host.as_deref().unwrap_or("(default only)"));

    if options.modules.is_empty() {
        output::keyval("Extra modules", "none");
    } else {
        output::keyval("Extra modules", &options.modules.join(", "));
    }

    output::keyval(
        "Backends in config",
        &config.get_backends().len().to_string(),
    );
}

pub(super) fn render_context_verbose(config_path: &Path, config: &MergedConfig) {
    output::header("Verbose Context");
    output::keyval("Config file", &config_path.display().to_string());
    output::keyval("Declared packages", &config.packages.len().to_string());
    output::keyval(
        "Known backends",
        &super::load_known_backends(config).len().to_string(),
    );
}

pub(super) fn render_install_remove_reason_block(
    action_word: &str,
    marker: char,
    items: &[PackageId],
) {
    if items.is_empty() {
        return;
    }

    output::separator();
    output::info(&format!("Why these will be {}:", action_word));
    for pkg in items.iter().take(15) {
        println!("  {} {}", marker, pkg);
    }
    if items.len() > 15 {
        println!("  ... and {} more", items.len() - 15);
    }
}

pub(super) fn render_backend_missing_suggestions() {
    output::info("Try one of these:");
    output::indent(
        &format!("• {}", project_identity::cli_with("info <package-name>")),
        1,
    );
    output::indent(
        &format!(
            "• {}",
            project_identity::cli_with("info <backend>:<package>")
        ),
        1,
    );
    output::indent(
        &format!(
            "• {}",
            project_identity::cli_with("info backend:<backend-name>")
        ),
        1,
    );
    output::indent(
        &format!("• {}", project_identity::cli_with("info <module-name>")),
        1,
    );
    output::indent(
        &format!("• {}", project_identity::cli_with("info --plan")),
        1,
    );
}
