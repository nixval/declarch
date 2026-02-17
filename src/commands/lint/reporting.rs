use super::{LintIssue, Severity};
use crate::ui as output;

pub(super) fn count_issues(issues: &[LintIssue]) -> (usize, usize) {
    let mut warn_count = 0;
    let mut err_count = 0;
    for issue in issues {
        match issue.severity {
            Severity::Warning => warn_count += 1,
            Severity::Error => err_count += 1,
        }
    }
    (warn_count, err_count)
}

pub(super) fn display_issues(issues: &[LintIssue]) -> (usize, usize) {
    let mut warn_count = 0;
    let mut err_count = 0;

    for issue in issues {
        let where_str = issue
            .file
            .as_ref()
            .map(|p| format!(" [{}]", p.display()))
            .unwrap_or_default();

        match issue.severity {
            Severity::Warning => {
                warn_count += 1;
                output::warning(&format!("{}{}", issue.message, where_str));
            }
            Severity::Error => {
                err_count += 1;
                output::error(&format!("{}{}", issue.message, where_str));
            }
        }
    }

    (warn_count, err_count)
}
