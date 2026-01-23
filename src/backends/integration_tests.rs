use crate::backends::{GenericManager, get_builtin_backends};
use crate::core::types::Backend;
use crate::packages::traits::PackageManager;

#[test]
fn test_npm_backend_config_exists() {
    let backends = get_builtin_backends();
    assert!(backends.contains_key("npm"));

    let npm_config = &backends["npm"];
    assert_eq!(npm_config.name, "npm");
    assert_eq!(
        npm_config.list_format,
        crate::backends::config::OutputFormat::Json
    );
}

#[test]
fn test_all_generic_backends_configured() {
    let backends = get_builtin_backends();

    // Verify all generic backends have configs
    let expected_backends = ["npm", "yarn", "pnpm", "bun", "pip", "cargo", "brew"];

    for backend in expected_backends {
        assert!(
            backends.contains_key(backend),
            "Missing backend: {}",
            backend
        );
    }
}

#[test]
fn test_generic_manager_creation() {
    let backends = get_builtin_backends();
    let npm_config = backends["npm"].clone();

    let manager = GenericManager::from_config(npm_config, Backend::Npm, false);

    assert_eq!(manager.backend_type(), Backend::Npm);
}

#[test]
fn test_npm_list_format() {
    let backends = get_builtin_backends();
    let npm_config = &backends["npm"];

    // Verify npm config has correct JSON parsing settings
    assert_eq!(npm_config.list_json_path, Some("dependencies".to_string()));
    assert_eq!(npm_config.list_name_key, Some("name".to_string()));
    assert_eq!(npm_config.list_version_key, Some("version".to_string()));
}

#[test]
fn test_pip_multiple_binaries() {
    let backends = get_builtin_backends();
    let pip_config = &backends["pip"];

    // pip should try pip3 first, then pip
    match &pip_config.binary {
        crate::backends::config::BinarySpecifier::Multiple(binaries) => {
            assert_eq!(binaries[0], "pip3");
            assert_eq!(binaries[1], "pip");
        }
        _ => panic!("pip should have Multiple binary specifier"),
    }
}
