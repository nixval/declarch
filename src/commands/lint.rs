use crate::config::kdl::parse_kdl_content_with_path;
use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use kdl::KdlDocument;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LintOptions {
    pub strict: bool,
    pub fix: bool,
    pub mode: LintMode,
    pub backend: Option<String>,
    pub diff: bool,
    pub benchmark: bool,
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

pub fn run(options: LintOptions) -> Result<()> {
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

    if options.fix {
        apply_safe_fixes(&lint_files)?;
    }

    let mut issues = Vec::new();

    collect_merged_issues(&merged, &mut issues);
    for file in &lint_files {
        collect_file_issues(file, &mut issues)?;
    }

    output::header("Lint Report");
    output::keyval("Files checked", &lint_files.len().to_string());
    output::keyval("Total issues", &issues.len().to_string());

    let (warn_count, err_count) = display_issues(&issues);
    output::keyval("Warnings", &warn_count.to_string());
    output::keyval("Errors", &err_count.to_string());

    if err_count > 0 {
        output::info("Tip: run `declarch lint --fix` for safe autofixes first.");
        return Err(DeclarchError::ConfigError(format!(
            "Lint failed with {} error(s)",
            err_count
        )));
    }

    if options.strict && warn_count > 0 {
        output::info("Strict mode treats warnings as blocking.");
        return Err(DeclarchError::ConfigError(format!(
            "Lint strict mode failed with {} warning(s)",
            warn_count
        )));
    }

    if issues.is_empty() {
        output::success("No lint issues found");
    } else {
        output::info("Use `declarch lint --fix` to apply safe fixes where available.");
        output::success("Lint completed");
    }

    Ok(())
}

fn collect_merged_issues(merged: &MergedConfig, issues: &mut Vec<LintIssue>) {
    let policy = merged.policy.as_ref();
    let duplicates = merged.get_duplicates();
    if !duplicates.is_empty() {
        let duplicate_as_error = policy.is_some_and(|p| p.duplicate_is_error());
        for (pkg, sources) in duplicates {
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

    let conflicts = merged.get_cross_backend_conflicts();
    if !conflicts.is_empty() {
        let conflict_as_error = policy.is_some_and(|p| p.conflict_is_error());
        for (pkg_name, backends) in conflicts {
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

fn resolve_import_path(base_file: &Path, import: &str) -> Result<PathBuf> {
    let parent = base_file.parent().ok_or_else(|| {
        DeclarchError::Other(format!(
            "Cannot determine parent directory for {}",
            base_file.display()
        ))
    })?;

    let candidate = if import.starts_with("~/") {
        crate::utils::paths::expand_home(Path::new(import))?
    } else if import.starts_with('/') {
        PathBuf::from(import)
    } else {
        parent.join(import)
    };

    if candidate.exists() || candidate.extension().is_some() {
        Ok(candidate)
    } else {
        Ok(candidate.with_extension("kdl"))
    }
}

fn display_issues(issues: &[LintIssue]) -> (usize, usize) {
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

fn collect_lint_files(config_path: &Path, modules: &[String]) -> Result<Vec<PathBuf>> {
    let mut files: BTreeSet<PathBuf> = BTreeSet::new();
    discover_lint_files_recursive(config_path, &mut files)?;

    for module in modules {
        let module_path = resolve_module_path(module)?;
        discover_lint_files_recursive(&module_path, &mut files)?;
    }

    Ok(files.into_iter().collect())
}

fn discover_lint_files_recursive(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !files.insert(canonical.clone()) {
        return Ok(());
    }

    if !canonical.exists() {
        return Ok(());
    }

    let content = match fs::read_to_string(&canonical) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };

    let file_str = canonical.display().to_string();
    let raw = match parse_kdl_content_with_path(&content, Some(&file_str)) {
        Ok(raw) => raw,
        Err(_) => return Ok(()),
    };

    for import in raw.imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            continue;
        }

        let resolved = resolve_import_path(&canonical, &import)?;
        if resolved.exists() {
            discover_lint_files_recursive(&resolved, files)?;
        }
    }

    for import in raw.backend_imports {
        if import
            .replace('\\', "/")
            .split('/')
            .any(|part| part == "..")
        {
            continue;
        }

        let resolved = resolve_import_path(&canonical, &import)?;
        if resolved.exists() {
            discover_lint_files_recursive(&resolved, files)?;
        }
    }

    Ok(())
}

fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
    selectors: &LoadSelectors,
) -> Result<MergedConfig> {
    let mut merged = loader::load_root_config_with_selectors(config_path, selectors)?;

    for module_name in extra_modules {
        let final_path = resolve_module_path(module_name)?;
        output::info(&format!("  Loading module: {}", final_path.display()));

        let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}

fn resolve_module_path(module_name: &str) -> Result<PathBuf> {
    if let Ok(path) = paths::module_file(module_name)
        && path.exists()
    {
        return Ok(path);
    }

    let direct = PathBuf::from(module_name);
    if direct.exists() {
        return Ok(direct);
    }

    Err(DeclarchError::Other(format!(
        "Module not found: {}",
        module_name
    )))
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

#[cfg(test)]
mod tests {
    use super::{discover_lint_files_recursive, sort_import_lines};
    use std::collections::BTreeSet;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn sort_import_lines_reorders_and_dedupes() {
        let input = r#"
imports {
  "b.kdl"
  "a.kdl"
  "a.kdl"
}
"#;

        let output = sort_import_lines(input);
        assert!(output.contains("\"a.kdl\""));
        assert!(output.contains("\"b.kdl\""));
        assert_eq!(output.matches("\"a.kdl\"").count(), 1);
    }

    #[test]
    fn discover_lint_files_traverses_nested_imports() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path().join("declarch.kdl");
        let modules_dir = dir.path().join("modules");
        fs::create_dir_all(&modules_dir).expect("mkdir");
        let base = modules_dir.join("base.kdl");
        let nested = modules_dir.join("nested.kdl");

        fs::write(
            &root,
            r#"
imports {
  "modules/base.kdl"
}
"#,
        )
        .expect("write root");
        fs::write(
            &base,
            r#"
imports {
  "nested.kdl"
}
pkg { aur { bat } }
"#,
        )
        .expect("write base");
        fs::write(&nested, "pkg { aur { ripgrep } }\n").expect("write nested");

        let mut files = BTreeSet::new();
        discover_lint_files_recursive(&root, &mut files).expect("discover");

        let as_strings: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
        assert_eq!(as_strings.len(), 3);
        assert!(as_strings.iter().any(|p| p.ends_with("declarch.kdl")));
        assert!(as_strings.iter().any(|p| p.ends_with("base.kdl")));
        assert!(as_strings.iter().any(|p| p.ends_with("nested.kdl")));
    }
}
