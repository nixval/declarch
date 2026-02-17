use super::*;

#[test]
fn test_parse_simple_backend() {
    let kdl = r#"
            backend "test" {
                binary "test-manager"
                list "test list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "test install {packages}"
                remove "test remove {packages}"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.list_cmd, Some("test list".to_string()));
    assert_eq!(config.install_cmd, "test install {packages}");
    assert_eq!(
        config.remove_cmd,
        Some("test remove {packages}".to_string())
    );
}

#[test]
fn test_parse_json_backend() {
    let kdl = r#"
            backend "npm" {
                binary "npm"
                list "npm list -g --json" {
                    format "json"
                    json_path "dependencies"
                    name_key "name"
                    version_key "version"
                }
                install "npm install -g {packages}"
                remove "npm uninstall -g {packages}"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    assert_eq!(config.name, "npm");
    assert_eq!(config.list_format, OutputFormat::Json);
    assert_eq!(config.list_json_path, Some("dependencies".to_string()));
    assert_eq!(config.list_name_key, Some("name".to_string()));
}

#[test]
fn test_parse_multiple_binaries() {
    let kdl = r#"
            backend "multi" {
                binary "cmd1" "cmd2" "cmd3"
                list "test list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "test install {packages}"
                remove "test remove {packages}"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    match config.binary {
        BinarySpecifier::Multiple(binaries) => {
            assert_eq!(binaries.len(), 3);
            assert_eq!(binaries[0], "cmd1");
            assert_eq!(binaries[1], "cmd2");
            assert_eq!(binaries[2], "cmd3");
        }
        _ => panic!("Expected Multiple binaries"),
    }
}

#[test]
fn test_parse_with_env() {
    let kdl = r#"
            backend "custom" {
                binary "custom"
                list "custom list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "custom install {packages}"
                remove "custom remove {packages}"
                env CACHE="/tmp/cache" DEBUG="1"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    assert!(config.preinstall_env.is_some());
    let env = config.preinstall_env.as_ref().unwrap();
    assert_eq!(env.get("CACHE"), Some(&"/tmp/cache".to_string()));
    assert_eq!(env.get("DEBUG"), Some(&"1".to_string()));
}

#[test]
fn test_parse_needs_sudo() {
    let kdl = r#"
            backend "sudo-test" {
                binary "apt"
                list "apt list" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "apt install {packages}"
                remove "apt remove {packages}"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let mut config = parse_backend_node(node).unwrap();

    config.needs_sudo = true;
    assert!(config.needs_sudo);
}

#[test]
fn test_parse_supported_os() {
    let kdl = r#"
            backend "cross" {
                binary "brew"
                list "brew list --versions" {
                    format "whitespace"
                    name_col 0
                    version_col 1
                }
                install "brew install {packages}"
                remove "brew uninstall {packages}"
                platforms "linux" "macos"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    assert_eq!(
        config.supported_os,
        Some(vec!["linux".to_string(), "macos".to_string()])
    );
}

#[test]
fn test_parse_prefer_list_for_local_search() {
    let kdl = r#"
            backend "npm" {
                binary "npm"
                list "npm list -g --json" {
                    format "json_object_keys"
                    json_path "dependencies"
                    version_key "version"
                }
                install "npm install -g {packages}"
                prefer_list_for_local_search "true"
            }
        "#;

    let doc = KdlDocument::parse(kdl).unwrap();
    let node = doc.nodes().first().unwrap();
    let config = parse_backend_node(node).unwrap();

    assert!(config.prefer_list_for_local_search);
}

#[test]
fn test_validate_missing_list_cmd() {
    let config = BackendConfig {
        name: "test".to_string(),
        list_cmd: None,
        install_cmd: "test install {packages}".to_string(),
        remove_cmd: Some("test remove {packages}".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = validate_backend_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_json_missing_name_key() {
    let config = BackendConfig {
        name: "test".to_string(),
        list_cmd: Some("test list {binary}".to_string()),
        list_format: OutputFormat::Json,
        install_cmd: "test install {packages}".to_string(),
        remove_cmd: Some("test remove {packages}".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = validate_backend_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_regex_missing_regex() {
    let config = BackendConfig {
        name: "test".to_string(),
        list_cmd: Some("test list {binary}".to_string()),
        list_format: OutputFormat::Regex,
        install_cmd: "test install {packages}".to_string(),
        remove_cmd: Some("test remove {packages}".to_string()),
        fallback: None,
        ..Default::default()
    };

    let result = validate_backend_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_missing_remove_cmd() {
    let config = BackendConfig {
        name: "test".to_string(),
        list_cmd: Some("test list".to_string()),
        install_cmd: "test install {packages}".to_string(),
        remove_cmd: None,
        fallback: None,
        ..Default::default()
    };

    let result = validate_backend_config(&config);
    assert!(result.is_ok());
}
