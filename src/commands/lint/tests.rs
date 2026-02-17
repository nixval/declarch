use super::{file_graph, sort_import_lines, state_ops};
use crate::core::types::Backend;
use crate::project_identity;
use crate::state::types::PackageState;
use chrono::Utc;
use std::collections::{BTreeSet, HashMap};
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
    let root = dir.path().join(project_identity::CONFIG_FILE_BASENAME);
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
    file_graph::discover_lint_files_recursive(&root, &mut files).expect("discover");

    let as_strings: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
    assert_eq!(as_strings.len(), 3);
    assert!(
        as_strings
            .iter()
            .any(|p| p.ends_with(project_identity::CONFIG_FILE_BASENAME))
    );
    assert!(as_strings.iter().any(|p| p.ends_with("base.kdl")));
    assert!(as_strings.iter().any(|p| p.ends_with("nested.kdl")));
}

fn pkg_state(backend: &str, name: &str) -> PackageState {
    PackageState {
        backend: Backend::from(backend),
        config_name: name.to_string(),
        provides_name: name.to_string(),
        actual_package_name: None,
        installed_at: Utc::now(),
        version: None,
        install_reason: Some("declared".to_string()),
        source_module: None,
        last_seen_at: None,
        backend_meta: None,
    }
}

#[test]
fn resolve_state_remove_keys_prefixed_id() {
    let mut packages: HashMap<String, PackageState> = HashMap::new();
    packages.insert("soar:firefox".to_string(), pkg_state("soar", "firefox"));

    let keys =
        state_ops::resolve_state_remove_keys(&packages, &["soar:firefox".to_string()], None, false)
            .expect("resolve keys");
    assert_eq!(keys, vec!["soar:firefox".to_string()]);
}

#[test]
fn resolve_state_remove_keys_backend_filter_plain_name() {
    let mut packages: HashMap<String, PackageState> = HashMap::new();
    packages.insert("aur:firefox".to_string(), pkg_state("aur", "firefox"));
    packages.insert("soar:firefox".to_string(), pkg_state("soar", "firefox"));

    let keys = state_ops::resolve_state_remove_keys(
        &packages,
        &["firefox".to_string()],
        Some("soar"),
        false,
    )
    .expect("resolve keys");
    assert_eq!(keys, vec!["soar:firefox".to_string()]);
}

#[test]
fn resolve_state_remove_keys_all_for_backend() {
    let mut packages: HashMap<String, PackageState> = HashMap::new();
    packages.insert("soar:firefox".to_string(), pkg_state("soar", "firefox"));
    packages.insert("soar:bat".to_string(), pkg_state("soar", "bat"));
    packages.insert("aur:bat".to_string(), pkg_state("aur", "bat"));

    let keys = state_ops::resolve_state_remove_keys(&packages, &[], Some("soar"), true)
        .expect("resolve backend keys");
    assert_eq!(
        keys,
        vec!["soar:bat".to_string(), "soar:firefox".to_string()]
    );
}

#[test]
fn resolve_state_remove_keys_ambiguous_plain_name_errors() {
    let mut packages: HashMap<String, PackageState> = HashMap::new();
    packages.insert("aur:firefox".to_string(), pkg_state("aur", "firefox"));
    packages.insert("soar:firefox".to_string(), pkg_state("soar", "firefox"));

    let err =
        state_ops::resolve_state_remove_keys(&packages, &["firefox".to_string()], None, false)
            .expect_err("plain name should be ambiguous");
    assert!(err.to_string().contains("matched multiple backends"));
}
