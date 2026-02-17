use crate::config::kdl::parse_kdl_content_with_path;
use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use crate::utils::machine_output;
use crate::utils::paths;
use kdl::KdlDocument;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

mod file_graph;
mod reporting;
mod state_ops;
#[cfg(test)]
mod tests;

use file_graph::{collect_lint_files, load_config_with_modules, resolve_import_path};
use reporting::{count_issues, display_issues};
use state_ops::{collect_state_issues, handle_state_remove};

pub struct LintOptions {
    pub strict: bool,
    pub fix: bool,
    pub mode: LintMode,
    pub backend: Option<String>,
    pub diff: bool,
    pub benchmark: bool,
    pub repair_state: bool,
    pub state_rm: Vec<String>,
    pub state_rm_backend: Option<String>,
    pub state_rm_all: bool,
    pub dry_run: bool,
    pub yes: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
    pub verbose: bool,
    pub profile: Option<String>,
    pub host: Option<String>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintMode {
    All,
    Validate,
    Duplicates,
    Conflicts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct LintIssue {
    severity: Severity,
    file: Option<PathBuf>,
    message: String,
}

impl LintIssue {
    fn warning(file: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            file,
            message: message.into(),
        }
    }

    fn error(file: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            file,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct LintIssueOut {
    severity: String,
    file: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct LintReportOut {
    mode: String,
    files_checked: usize,
    total_issues: usize,
    warnings_count: usize,
    errors_count: usize,
    issues: Vec<LintIssueOut>,
}

pub fn run(options: LintOptions) -> Result<()> {
    let start_time = std::time::Instant::now();
    if handle_state_remove(&options)? {
        return Ok(());
    }

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(DeclarchError::ConfigNotFound { path: config_path });
    }

    let selectors = LoadSelectors {
        profile: options.profile.clone(),
        host: options.host.clone(),
    };

    let merged = if options.modules.is_empty() {
        loader::load_root_config_with_selectors(&config_path, &selectors)?
    } else {
        load_config_with_modules(&config_path, &options.modules, &selectors)?
    };

    let lint_files = collect_lint_files(&config_path, &options.modules)?;

    let machine_mode = matches!(options.output_version.as_deref(), Some("v1"))
        && matches!(options.format.as_deref(), Some("json" | "yaml"));

    if options.fix {
        apply_safe_fixes(&lint_files)?;
    }

    if options.repair_state {
        let report = crate::state::io::repair_state_packages()?;
        if !machine_mode {
            output::header("State Repair");
            output::keyval("Entries before", &report.total_before.to_string());
            output::keyval("Entries after", &report.total_after.to_string());
            output::keyval(
                "Removed (empty name)",
                &report.removed_empty_name.to_string(),
            );
            output::keyval(
                "Removed (duplicates)",
                &report.removed_duplicates.to_string(),
            );
            output::keyval("Rekeyed entries", &report.rekeyed_entries.to_string());
            output::keyval("Normalized fields", &report.normalized_fields.to_string());
        }
    }

    if options.diff && !machine_mode {
        show_diff(&merged)?;
    }

    let mut issues = Vec::new();

    match options.mode {
        LintMode::All => {
            collect_duplicate_issues(&merged, options.backend.as_deref(), &mut issues);
            collect_conflict_issues(&merged, options.backend.as_deref(), &mut issues);
            collect_misc_merged_issues(&merged, &mut issues);
            for file in &lint_files {
                collect_file_issues(file, &mut issues)?;
            }
            collect_state_issues(&mut issues)?;
        }
        LintMode::Validate => {
            for file in &lint_files {
                collect_file_issues(file, &mut issues)?;
            }
        }
        LintMode::Duplicates => {
            collect_duplicate_issues(&merged, options.backend.as_deref(), &mut issues);
        }
        LintMode::Conflicts => {
            collect_conflict_issues(&merged, options.backend.as_deref(), &mut issues);
        }
    }

    let (warn_count, err_count) = if machine_mode {
        count_issues(&issues)
    } else {
        output::header("Lint Report");
        output::keyval("Files checked", &lint_files.len().to_string());
        output::keyval("Total issues", &issues.len().to_string());

        let (warn_count, err_count) = display_issues(&issues);
        output::keyval("Warnings", &warn_count.to_string());
        output::keyval("Errors", &err_count.to_string());
        if options.verbose {
            output::keyval("Mode", &format!("{:?}", options.mode));
            output::keyval(
                "Backend filter",
                options.backend.as_deref().unwrap_or("(none)"),
            );
        }
        (warn_count, err_count)
    };

    if machine_mode {
        let issues_out = issues
            .iter()
            .map(|issue| LintIssueOut {
                severity: match issue.severity {
                    Severity::Warning => "warning".to_string(),
                    Severity::Error => "error".to_string(),
                },
                file: issue.file.as_ref().map(|p| p.display().to_string()),
                message: issue.message.clone(),
            })
            .collect::<Vec<_>>();

        let report = LintReportOut {
            mode: format!("{:?}", options.mode).to_lowercase(),
            files_checked: lint_files.len(),
            total_issues: issues.len(),
            warnings_count: warn_count,
            errors_count: err_count,
            issues: issues_out,
        };
        machine_output::emit_v1(
            "lint",
            report,
            Vec::new(),
            Vec::new(),
            options.format.as_deref().unwrap_or("json"),
        )?;
    }

    if err_count > 0 {
        if !machine_mode {
            output::info(&format!(
                "Tip: run `{}` for safe autofixes first.",
                project_identity::cli_with("lint --fix")
            ));
        }
        return Err(DeclarchError::ConfigError(format!(
            "Lint failed with {} error(s)",
            err_count
        )));
    }

    if options.strict && warn_count > 0 {
        if !machine_mode {
            output::info("Strict mode treats warnings as blocking.");
        }
        return Err(DeclarchError::ConfigError(format!(
            "Lint strict mode failed with {} warning(s)",
            warn_count
        )));
    }

    if !machine_mode {
        if issues.is_empty() {
            output::success("No lint issues found");
        } else {
            output::info(&format!(
                "Use `{}` to apply safe fixes where available.",
                project_identity::cli_with("lint --fix")
            ));
            output::success("Lint completed");
        }
    }

    if options.benchmark && !machine_mode {
        output::keyval("Elapsed", &format!("{:?}", start_time.elapsed()));
    }

    Ok(())
}

fn collect_duplicate_issues(
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

fn collect_conflict_issues(
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

fn collect_misc_merged_issues(merged: &MergedConfig, issues: &mut Vec<LintIssue>) {
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

fn collect_file_issues(path: &Path, issues: &mut Vec<LintIssue>) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let file_str = path.display().to_string();

    // Parse KDL document to inspect top-level node names safely.
    let doc: KdlDocument = match content.parse() {
        Ok(doc) => doc,
        Err(e) => {
            issues.push(LintIssue::error(
                Some(path.to_path_buf()),
                format!("Invalid KDL syntax: {}", e),
            ));
            return Ok(());
        }
    };

    for node in doc.nodes() {
        let name = node.name().value();
        if name == "packages" || name.starts_with("packages:") {
            issues.push(LintIssue::warning(
                Some(path.to_path_buf()),
                "Deprecated syntax block 'packages' detected. Use 'pkg' syntax",
            ));
        }
    }

    let raw = match parse_kdl_content_with_path(&content, Some(&file_str)) {
        Ok(raw) => raw,
        Err(e) => {
            issues.push(LintIssue::error(
                Some(path.to_path_buf()),
                format!("Failed to parse config content: {}", e),
            ));
            return Ok(());
        }
    };

    for import in raw.imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            issues.push(LintIssue::warning(
                Some(path.to_path_buf()),
                format!("Import contains path traversal '..': {}", import),
            ));
            continue;
        }

        let resolved = resolve_import_path(path, &import)?;
        if !resolved.exists() {
            issues.push(LintIssue::warning(
                Some(path.to_path_buf()),
                format!(
                    "Unresolved import: '{}' (resolved: {})",
                    import,
                    resolved.display()
                ),
            ));
        }
    }

    for import in raw.backend_imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            issues.push(LintIssue::warning(
                Some(path.to_path_buf()),
                format!("Backend import contains path traversal '..': {}", import),
            ));
            continue;
        }

        let resolved = resolve_import_path(path, &import)?;
        if !resolved.exists() {
            issues.push(LintIssue::warning(
                Some(path.to_path_buf()),
                format!(
                    "Unresolved backend import: '{}' (resolved: {})",
                    import,
                    resolved.display()
                ),
            ));
        }
    }

    Ok(())
}

fn apply_safe_fixes(files: &[PathBuf]) -> Result<()> {
    output::header("Applying Lint Fixes");

    for file in files {
        let content = fs::read_to_string(file)?;
        let fixed = sort_import_lines(&content);
        if fixed != content {
            fs::write(file, fixed)?;
            output::success(&format!("Fixed {}", file.display()));
        }
    }

    Ok(())
}

fn show_diff(config: &MergedConfig) -> Result<()> {
    use crate::core::types::PackageId;

    let state_path = crate::state::io::get_state_path()?;
    if !state_path.exists() {
        output::info("Packages to install:");
        for pkg_id in config.packages.keys() {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
        return Ok(());
    }

    let state = crate::state::io::load_state()?;
    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
    let mut state_set: HashSet<PackageId> = HashSet::new();

    for pkg_state in state.packages.values() {
        state_set.insert(PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        });
    }

    let to_install: Vec<_> = config_set.difference(&state_set).cloned().collect();
    let to_remove: Vec<_> = state_set.difference(&config_set).cloned().collect();

    if to_install.is_empty() && to_remove.is_empty() {
        output::info("No changes planned");
        return Ok(());
    }

    if !to_install.is_empty() {
        output::info("To install:");
        for pkg_id in &to_install {
            println!("  + {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    if !to_remove.is_empty() {
        output::info("To remove:");
        for pkg_id in &to_remove {
            println!("  - {} {}", pkg_id.backend, pkg_id.name);
        }
    }

    Ok(())
}

fn sort_import_lines(content: &str) -> String {
    let mut result = String::new();
    let mut imports: Vec<String> = Vec::new();
    let mut in_imports = false;
    let mut imports_indent = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "imports {" || trimmed.starts_with("imports {") {
            in_imports = true;
            imports_indent = line[..line.find("imports").unwrap_or(0)].to_string();
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_imports && trimmed == "}" {
            imports.sort();
            imports.dedup();
            for entry in &imports {
                result.push_str(&format!("{}  {}\n", imports_indent, entry));
            }
            imports.clear();
            in_imports = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_imports && trimmed.starts_with('"') {
            imports.push(trimmed.to_string());
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    let trimmed = result.trim_end();
    format!("{}\n", trimmed)
}
