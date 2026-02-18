use crate::cli::args::LintMode;
use crate::commands;
use crate::error::{DeclarchError, Result};

pub(super) fn map_lint_mode(mode: &LintMode) -> commands::lint::LintMode {
    match mode {
        LintMode::All => commands::lint::LintMode::All,
        LintMode::Validate => commands::lint::LintMode::Validate,
        LintMode::Duplicates => commands::lint::LintMode::Duplicates,
        LintMode::Conflicts => commands::lint::LintMode::Conflicts,
    }
}

pub(super) fn list_to_optional_vec(values: &[String]) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values.to_vec())
    }
}

pub(super) fn parse_limit_option(limit: Option<&str>) -> Result<Option<usize>> {
    match limit {
        None => Ok(Some(10)),
        Some("all") | Some("0") => Ok(None),
        Some(raw) => raw.parse::<usize>().map(Some).map_err(|_| {
            DeclarchError::Other(format!(
                "Invalid --limit value '{}'. Use a non-negative integer, 0, or 'all'.",
                raw
            ))
        }),
    }
}
