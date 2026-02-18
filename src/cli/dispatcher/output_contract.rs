use crate::cli::args::{Cli, Command};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;

pub(super) fn validate_machine_output_contract(args: &Cli) -> Result<()> {
    if let Some(version) = args.global.output_version.as_deref() {
        if version != "v1" {
            return Err(DeclarchError::Other(format!(
                "Unsupported output contract version '{}'. Supported: v1",
                version
            )));
        }

        match args.global.format.as_deref() {
            Some("json") | Some("yaml") => {}
            Some(other) => {
                return Err(DeclarchError::Other(format!(
                    "--output-version v1 requires --format json|yaml (got '{}')",
                    other
                )));
            }
            None => {
                output::warning(
                    "--output-version v1 is set without --format; output remains human-oriented.",
                );
            }
        }

        if !supports_v1_contract(args) {
            return Err(DeclarchError::Other(format!(
                "This command does not support --output-version v1 yet.\nSupported now: `{}`, `{}`, `{}`, `{}`, `{}`.",
                project_identity::cli_with("info"),
                project_identity::cli_with("info --list"),
                project_identity::cli_with("lint"),
                project_identity::cli_with("search"),
                project_identity::cli_with("--dry-run sync"),
            )));
        }
    }

    Ok(())
}

fn supports_v1_contract(args: &Cli) -> bool {
    match &args.command {
        Some(Command::Lint { .. }) => true,
        Some(Command::Search { .. }) => true,
        Some(Command::Sync { command: None, .. }) => args.global.dry_run,
        Some(Command::Info {
            doctor,
            plan,
            query,
            ..
        }) => !*doctor && !*plan && query.is_none(),
        _ => false,
    }
}
