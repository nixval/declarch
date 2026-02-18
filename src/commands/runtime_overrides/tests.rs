use super::*;
use std::collections::HashMap;

#[test]
fn apply_runtime_backend_overrides_applies_options_env_and_sources() {
    let mut backend = BackendConfig {
        name: "paru".to_string(),
        install_cmd: "paru -S {packages}".to_string(),
        noconfirm_flag: Some("--noconfirm".to_string()),
        ..Default::default()
    };

    let mut merged = loader::MergedConfig::default();
    let mut options = HashMap::new();
    options.insert("noconfirm_flag".to_string(), "--yes".to_string());
    options.insert("needs_sudo".to_string(), "true".to_string());
    merged.backend_options.insert("paru".to_string(), options);
    merged.env.insert(
        "paru".to_string(),
        vec!["A=1".to_string(), "B=2".to_string()],
    );
    merged.package_sources.insert(
        "paru".to_string(),
        vec!["core".to_string(), "extra".to_string()],
    );

    apply_runtime_backend_overrides(&mut backend, "paru", &merged);

    assert_eq!(backend.noconfirm_flag.as_deref(), Some("--yes"));
    assert!(backend.needs_sudo);
    let env = backend.preinstall_env.expect("env set");
    assert_eq!(env.get("A").map(String::as_str), Some("1"));
    assert_eq!(env.get("B").map(String::as_str), Some("2"));
    assert_eq!(
        backend.package_sources,
        Some(vec!["core".to_string(), "extra".to_string()])
    );
}
