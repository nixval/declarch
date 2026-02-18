use super::LintIssue;
use crate::config::kdl::parse_kdl_content_with_path;
use crate::error::Result;
use crate::ui as output;
use kdl::KdlDocument;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn collect_file_issues(path: &Path, issues: &mut Vec<LintIssue>) -> Result<()> {
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

        let resolved = super::file_graph::resolve_import_path(path, &import)?;
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

        let resolved = super::file_graph::resolve_import_path(path, &import)?;
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

pub(super) fn apply_safe_fixes(files: &[PathBuf]) -> Result<()> {
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

pub(super) fn sort_import_lines(content: &str) -> String {
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
