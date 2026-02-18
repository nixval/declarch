use super::*;

#[test]
fn test_templates_not_empty() {
    assert!(!default_config().is_empty());
    assert!(!default_host("laptop").is_empty());
    assert!(!default_module("tools").is_empty());
}

#[test]
fn test_default_host_uses_declarch_backends_block_only() {
    let tpl = default_host("laptop");
    assert!(tpl.contains("backends {"));
    assert!(!tpl.contains("backends.kdl"));
}
