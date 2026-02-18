use super::LintIssue;
use crate::config::loader::MergedConfig;

pub(super) fn collect_duplicate_issues(
    merged: &MergedConfig,
    backend_filter: Option<&str>,
    issues: &mut Vec<LintIssue>,
) {
    let policy = merged.policy.as_ref();
    let duplicates = merged.get_duplicates();
    let duplicate_as_error = policy.is_some_and(|p| p.duplicate_is_error());
    for (pkg, sources) in duplicates {
        if let Some(filter) = backend_filter
            && pkg.backend.name() != filter
        {
            continue;
        }
        let msg = format!(
            "Duplicate declaration: {} appears in {} source file(s)",
            pkg,
            sources.len()
        );
        if duplicate_as_error {
            issues.push(LintIssue::error(None, msg));
        } else {
            issues.push(LintIssue::warning(None, msg));
        }
    }
}

pub(super) fn collect_conflict_issues(
    merged: &MergedConfig,
    backend_filter: Option<&str>,
    issues: &mut Vec<LintIssue>,
) {
    let policy = merged.policy.as_ref();
    let conflicts = merged.get_cross_backend_conflicts();
    let conflict_as_error = policy.is_some_and(|p| p.conflict_is_error());
    for (pkg_name, backends) in conflicts {
        if let Some(filter) = backend_filter
            && !backends.iter().any(|b| b.name() == filter)
        {
            continue;
        }
        let backend_list = backends
            .into_iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let msg = format!(
            "Cross-backend conflict candidate: '{}' exists in [{}]",
            pkg_name, backend_list
        );
        if conflict_as_error {
            issues.push(LintIssue::error(None, msg));
        } else {
            issues.push(LintIssue::warning(None, msg));
        }
    }
}

pub(super) fn collect_misc_merged_issues(merged: &MergedConfig, issues: &mut Vec<LintIssue>) {
    let policy = merged.policy.as_ref();
    if merged.lifecycle_actions.is_some() && !merged.is_experimental_enabled("enable-hooks") {
        issues.push(LintIssue::warning(
            None,
            "Hooks configured but not executable: add experimental { \"enable-hooks\" } if intentional",
        ));
    }

    for pkg in merged.packages.keys() {
        if pkg.backend.to_string() == "default" {
            let msg = format!(
                "Package '{}' uses implicit 'default' backend (legacy syntax). Prefer explicit pkg{{ backend {{ ... }} }}",
                pkg.name
            );
            if policy.and_then(|p| p.require_backend).unwrap_or(false) {
                issues.push(LintIssue::error(None, msg));
            } else {
                issues.push(LintIssue::warning(None, msg));
            }
        }
    }
}
