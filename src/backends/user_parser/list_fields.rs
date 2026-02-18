use super::parse_utils::get_entry_string;
use crate::backends::config::{BackendConfig, OutputFormat};
use crate::error::{DeclarchError, Result};
use kdl::KdlNode;

pub(super) fn parse_list_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(get_entry_string)
        .ok_or_else(|| {
            DeclarchError::Other(
                "List command required. Usage: list \"command\" { ... }".to_string(),
            )
        })?;

    if cmd != "-" {
        config.list_cmd = Some(cmd.to_string());
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            match child_name {
                "format" => {
                    let format_str = child
                        .entries()
                        .first()
                        .and_then(get_entry_string)
                        .ok_or_else(|| {
                            DeclarchError::Other(
                                "Format value required. Usage: format json|whitespace|tsv|regex"
                                    .to_string(),
                            )
                        })?;

                    config.list_format = match format_str.as_str() {
                        "json" => OutputFormat::Json,
                        "json_lines" | "jsonl" | "ndjson" => OutputFormat::JsonLines,
                        "npm_json" => OutputFormat::NpmJson,
                        "json_object_keys" => OutputFormat::JsonObjectKeys,
                        "whitespace" => OutputFormat::SplitWhitespace,
                        "tsv" => OutputFormat::TabSeparated,
                        "regex" => OutputFormat::Regex,
                        _ => {
                            return Err(DeclarchError::Other(format!(
                                "Unknown format '{}'. Valid: json, json_lines, npm_json, json_object_keys, whitespace, tsv, regex",
                                format_str
                            )));
                        }
                    };
                }
                "json_path" => {
                    config.list_json_path = child.entries().first().and_then(get_entry_string);
                }
                "name_key" => {
                    config.list_name_key = child.entries().first().and_then(get_entry_string);
                }
                "version_key" => {
                    config.list_version_key = child.entries().first().and_then(get_entry_string);
                }
                "json" => {
                    if let Some(json_children) = child.children() {
                        for json_child in json_children.nodes() {
                            match json_child.name().value() {
                                "path" => {
                                    config.list_json_path = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_key" => {
                                    config.list_name_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "version_key" => {
                                    config.list_version_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "desc_key" => {}
                                _ => {}
                            }
                        }
                    }
                }
                "name_col" => {
                    config.list_name_col = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                "version_col" => {
                    config.list_version_col = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                "regex" => {
                    if let Some(regex_children) = child.children() {
                        for regex_child in regex_children.nodes() {
                            match regex_child.name().value() {
                                "pattern" => {
                                    config.list_regex = regex_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_group" => {
                                    config.list_regex_name_group =
                                        regex_child.entries().first().and_then(|entry| {
                                            entry
                                                .value()
                                                .as_string()
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .or_else(|| {
                                                    let val_str = entry.value().to_string();
                                                    val_str.parse::<usize>().ok()
                                                })
                                        });
                                }
                                "version_group" => {
                                    config.list_regex_version_group =
                                        regex_child.entries().first().and_then(|entry| {
                                            entry
                                                .value()
                                                .as_string()
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .or_else(|| {
                                                    let val_str = entry.value().to_string();
                                                    val_str.parse::<usize>().ok()
                                                })
                                        });
                                }
                                _ => {}
                            }
                        }
                    }
                    if config.list_regex.is_none()
                        && let Some(pattern) = child
                            .entries()
                            .first()
                            .and_then(|entry| entry.value().as_string())
                    {
                        config.list_regex = Some(pattern.to_string());
                    }
                }
                "pattern" | "regex_pat" | "myregex" => {
                    config.list_regex = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.list_regex_name_group = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                "version_group" => {
                    config.list_regex_version_group = child.entries().first().and_then(|entry| {
                        entry
                            .value()
                            .as_string()
                            .and_then(|s| s.parse::<usize>().ok())
                            .or_else(|| {
                                let val_str = entry.value().to_string();
                                val_str.parse::<usize>().ok()
                            })
                    });
                }
                _ => {}
            }
        }
    }

    Ok(())
}
