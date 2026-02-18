use super::{looks_like_module_query, parse_query};

#[test]
fn parse_query_backend_prefix() {
    let (backend, name) = parse_query("aur:bat");
    assert_eq!(backend.expect("backend").to_string(), "aur");
    assert_eq!(name, "bat");
}

#[test]
fn parse_query_plain_name() {
    let (backend, name) = parse_query("firefox");
    assert!(backend.is_none());
    assert_eq!(name, "firefox");
}

#[test]
fn parse_query_backend_keyword() {
    let (backend, name) = parse_query("backend:pnpm");
    assert_eq!(backend.expect("backend").to_string(), "backend");
    assert_eq!(name, "pnpm");
}

#[test]
fn detect_module_query_shape() {
    assert!(looks_like_module_query("system/base"));
    assert!(looks_like_module_query("modules/dev.kdl"));
    assert!(!looks_like_module_query("firefox"));
}
