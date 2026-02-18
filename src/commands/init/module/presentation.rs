use crate::config::kdl::parse_kdl_content;
use crate::constants::CONFIG_FILE_NAME;
use crate::error::{DeclarchError, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;

pub(super) fn display_module_meta(content: &str) {
    if let Ok(raw_config) = parse_kdl_content(content) {
        let meta = &raw_config.project_metadata;

        let has_meta = meta.title.is_some()
            || meta.description.is_some()
            || meta.author.is_some()
            || !meta.maintainers.is_empty()
            || meta.version.is_some()
            || !meta.tags.is_empty()
            || meta.url.is_some()
            || meta.homepage.is_some()
            || meta.license.is_some()
            || !meta.platforms.is_empty()
            || !meta.requires.is_empty()
            || meta.installation_guide.is_some();

        if has_meta {
            crate::ui::separator();
            println!("{}", "Module Information:".bold().cyan());

            if let Some(title) = &meta.title {
                println!("  {}", title.bold());
                println!();
            }

            if let Some(description) = &meta.description {
                println!("  {}", description.dimmed());
                println!();
            }

            let mut details = Vec::new();
            if let Some(author) = &meta.author {
                details.push(format!("Author: {}", author.yellow()));
            }
            if !meta.maintainers.is_empty() {
                details.push(format!(
                    "Maintainer: {}",
                    meta.maintainers.join(", ").yellow()
                ));
            }
            if let Some(version) = &meta.version {
                details.push(format!("Version: {}", version.green()));
            }
            if !meta.tags.is_empty() {
                details.push(format!("Tags: {}", meta.tags.join(", ").purple()));
            }
            if let Some(url) = &meta.url {
                details.push(format!("URL: {}", url.blue().underline()));
            }
            if let Some(homepage) = &meta.homepage {
                details.push(format!("Homepage: {}", homepage.blue().underline()));
            }
            if let Some(license) = &meta.license {
                details.push(format!("License: {}", license.normal()));
            }
            if !meta.platforms.is_empty() {
                details.push(format!("Platforms: {}", meta.platforms.join(", ").normal()));
            }
            if !meta.requires.is_empty() {
                details.push(format!("Requires: {}", meta.requires.join(", ").normal()));
            }
            if let Some(guide) = &meta.installation_guide {
                details.push(format!("Installation Guide: {}", guide.blue().underline()));
            }

            for detail in details {
                println!("  {}", detail);
            }

            crate::ui::separator();
        }
    }
}

pub(super) fn inject_import_to_root(
    config_path: &Path,
    import_path: &str,
    force: bool,
    yes: bool,
) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    let import_line = format!("    {:?}", import_path);

    let active_import_pattern = format!(r#"(?m)^\s+"{}"\s*$"#, regex::escape(import_path));
    if Regex::new(&active_import_pattern)
        .map(|re| re.is_match(&content))
        .unwrap_or(false)
    {
        crate::ui::info(&format!(
            "Module '{}' is already imported in config.",
            import_path
        ));
        return Ok(());
    }

    if !force
        && !yes
        && !crate::ui::prompt_yes_no(&format!(
            "Add '{}' to imports in {}?",
            import_path, CONFIG_FILE_NAME
        ))
    {
        crate::ui::info("Skipping auto-import. You can add it manually.");
        return Ok(());
    }

    let re = Regex::new(r#"(?m)^(\s*imports\s*\{)"#)
        .map_err(|e| DeclarchError::Other(format!("Invalid regex pattern: {}", e)))?;

    let new_content = if re.is_match(&content) {
        re.replace(&content, |caps: &regex::Captures| {
            format!("{}\n{}", &caps[0], import_line)
        })
        .to_string()
    } else {
        format!(
            "{}\n\nimports {{\n{}\n}}\n",
            content.trim_end(),
            import_line
        )
    };

    fs::write(config_path, new_content)?;
    Ok(())
}
