use super::parse_utils::get_entry_string;
use crate::backends::config::{BackendConfig, OutputFormat};
use crate::error::{DeclarchError, Result};
use kdl::KdlNode;

pub(super) fn parse_search_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "Search command required. Usage: search \"command\" { ... }".to_string(),
            )
        })?;

    if cmd == "-" {
        return Ok(());
    }

    config.search_cmd = Some(cmd.to_string());

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

                    config.search_format = Some(match format_str.as_str() {
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
                    });
                }
                "json_path" => {
                    config.search_json_path = child.entries().first().and_then(get_entry_string);
                }
                "name_key" => {
                    config.search_name_key = child.entries().first().and_then(get_entry_string);
                }
                "version_key" => {
                    config.search_version_key = child.entries().first().and_then(get_entry_string);
                }
                "desc_key" => {
                    config.search_desc_key = child.entries().first().and_then(get_entry_string);
                }
                "json" => {
                    if let Some(json_children) = child.children() {
                        for json_child in json_children.nodes() {
                            match json_child.name().value() {
                                "path" => {
                                    config.search_json_path = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_key" => {
                                    config.search_name_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "version_key" => {
                                    config.search_version_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "desc_key" => {
                                    config.search_desc_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "name_col" => {
                    config.search_name_col = child.entries().first().and_then(|entry| {
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
                "desc_col" => {
                    config.search_desc_col = child.entries().first().and_then(|entry| {
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
                    config.search_regex = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.search_regex_name_group = child.entries().first().and_then(|entry| {
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
                "desc_group" => {
                    config.search_regex_desc_group = child.entries().first().and_then(|entry| {
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

pub(super) fn parse_search_local_cmd(node: &KdlNode, config: &mut BackendConfig) -> Result<()> {
    let cmd = node
        .entries()
        .first()
        .and_then(|entry| entry.value().as_string())
        .ok_or_else(|| {
            DeclarchError::Other(
                "Search local command required. Usage: search_local \"command\" { ... }"
                    .to_string(),
            )
        })?
        .to_string();

    if cmd == "-" {
        return Ok(());
    }

    config.search_local_cmd = Some(cmd);

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

                    config.search_local_format = Some(match format_str.as_str() {
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
                    });
                }
                "json_path" => {
                    config.search_local_json_path =
                        child.entries().first().and_then(get_entry_string);
                }
                "name_key" => {
                    config.search_local_name_key =
                        child.entries().first().and_then(get_entry_string);
                }
                "version_key" => {
                    config.search_local_version_key =
                        child.entries().first().and_then(get_entry_string);
                }
                "json" => {
                    if let Some(json_children) = child.children() {
                        for json_child in json_children.nodes() {
                            match json_child.name().value() {
                                "path" => {
                                    config.search_local_json_path = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "name_key" => {
                                    config.search_local_name_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                "version_key" => {
                                    config.search_local_version_key = json_child
                                        .entries()
                                        .first()
                                        .and_then(|entry| entry.value().as_string())
                                        .map(|s| s.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "name_col" => {
                    config.search_local_name_col = child.entries().first().and_then(|entry| {
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
                    config.search_local_regex = child
                        .entries()
                        .first()
                        .and_then(|entry| entry.value().as_string())
                        .map(|s| s.to_string());
                }
                "name_group" => {
                    config.search_local_regex_name_group =
                        child.entries().first().and_then(|entry| {
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
