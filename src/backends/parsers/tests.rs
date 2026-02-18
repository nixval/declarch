use super::*;

#[test]
fn test_parse_whitespace() {
    let config = BackendConfig {
        list_name_col: Some(0),
        list_version_col: Some(1),
        fallback: None,
        ..Default::default()
    };

    let output = b"package1 1.0.0\npackage2 2.0.0\n";
    let result = parse_package_list(output, &config).expect("parse package list");

    assert_eq!(result.len(), 2);
    assert!(result.contains_key("package1"));
    assert!(result.contains_key("package2"));
    assert_eq!(result["package1"].version.as_deref(), Some("1.0.0"));
}
