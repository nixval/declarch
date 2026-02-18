use super::*;
use crate::project_identity;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_add_backend_to_declarch_success() {
    let content = r#"meta {
    title "Host"
}

backends {
    "backends/aur.kdl"
}
"#;

    let mut temp_file = NamedTempFile::new().expect("tempfile");
    temp_file
        .write_all(content.as_bytes())
        .expect("write content");

    let result = add_backend_to_declarch(temp_file.path(), "flatpak").expect("add backend");
    assert!(result);

    let new_content = fs::read_to_string(temp_file.path()).expect("read content");
    assert!(new_content.contains(r#""backends/flatpak.kdl""#));
    assert!(!new_content.contains(r#"\"backends/flatpak.kdl\""#));
}

#[test]
fn test_add_backend_to_declarch_no_block() {
    let content = r#"meta { title "Host" }"#;

    let mut temp_file = NamedTempFile::new().expect("tempfile");
    temp_file
        .write_all(content.as_bytes())
        .expect("write content");

    let result = add_backend_to_declarch(temp_file.path(), "flatpak").expect("add backend");
    assert!(!result);
}

#[test]
fn test_extract_backend_meta_maintainer_alias() {
    let content = r#"
backend "nala" {
    meta {
        title "Nala"
        maintained "declarch"
        kdl-maintainer "community-a" "community-b"
        homepage "https://example.com"
        install-guide "https://example.com/guide"
        requires "nala" "apt"
    }
    binary "nala"
    install "{binary} install {packages}"
}
"#
    .replace(
        "maintained \"declarch\"",
        &format!("maintained \"{}\"", project_identity::STABLE_PROJECT_ID),
    );
    let meta = extract_backend_meta(&content).expect("extract meta");
    assert_eq!(meta.title, "Nala");
    assert_eq!(
        meta.maintainers,
        vec![
            "community-a".to_string(),
            "community-b".to_string(),
            project_identity::STABLE_PROJECT_ID.to_string(),
        ]
    );
    assert_eq!(
        meta.installation_guide,
        Some("https://example.com/guide".to_string())
    );
    assert_eq!(meta.requires, vec!["apt".to_string(), "nala".to_string()]);
}
