use crate::backends::config::{BackendConfig, BinarySpecifier};
use std::collections::HashMap;

/// Get built-in backend configurations
pub fn get_builtin_backends() -> HashMap<String, BackendConfig> {
    let mut backends = HashMap::new();

    // === NPM Backend ===
    backends.insert("npm".to_string(), BackendConfig {
        name: "npm".to_string(),
        binary: BinarySpecifier::Single("npm".to_string()),
        list_cmd: "npm list -g --depth=0 --json 2>/dev/null".to_string(),
        install_cmd: "npm install -g {packages}".to_string(),
        remove_cmd: "npm uninstall -g {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Json,
        list_name_col: None,
        list_version_col: None,
        list_json_path: Some("dependencies".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: None,  // npm doesn't have a noconfirm flag
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    // === Yarn Backend ===
    backends.insert("yarn".to_string(), BackendConfig {
        name: "yarn".to_string(),
        binary: BinarySpecifier::Single("yarn".to_string()),
        list_cmd: "npm list -g --depth=0 --json 2>/dev/null".to_string(), // Use npm list (yarn global is messy)
        install_cmd: "yarn global add {packages}".to_string(),
        remove_cmd: "yarn global remove {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Json,
        list_name_col: None,
        list_version_col: None,
        list_json_path: Some("dependencies".to_string()),
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: None,
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    // === PNPM Backend ===
    backends.insert("pnpm".to_string(), BackendConfig {
        name: "pnpm".to_string(),
        binary: BinarySpecifier::Single("pnpm".to_string()),
        list_cmd: "pnpm list -g --depth=0 --json 2>/dev/null".to_string(),
        install_cmd: "pnpm add -g {packages}".to_string(),
        remove_cmd: "pnpm remove -g {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Json,
        list_name_col: None,
        list_version_col: None,
        list_json_path: Some("".to_string()), // Root array
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: None,
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    // === BUN Backend ===
    backends.insert("bun".to_string(), BackendConfig {
        name: "bun".to_string(),
        binary: BinarySpecifier::Single("bun".to_string()),
        list_cmd: "bun pm ls -g --json".to_string(),
        install_cmd: "bun install -g {packages}".to_string(),
        remove_cmd: "bun remove -g {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Json,
        list_name_col: None,
        list_version_col: None,
        list_json_path: Some("".to_string()), // Root array
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: None,
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    // === PIP Backend ===
    backends.insert("pip".to_string(), BackendConfig {
        name: "pip".to_string(),
        binary: BinarySpecifier::Multiple(vec!["pip3".to_string(), "pip".to_string()]),
        list_cmd: "pip list --format=json 2>/dev/null".to_string(),
        install_cmd: "pip install --break-system-packages {packages}".to_string(),
        remove_cmd: "pip uninstall -y {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Json,
        list_name_col: None,
        list_version_col: None,
        list_json_path: Some("".to_string()), // Root array
        list_name_key: Some("name".to_string()),
        list_version_key: Some("version".to_string()),
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: Some("-y".to_string()),
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    // === Cargo Backend ===
    backends.insert("cargo".to_string(), BackendConfig {
        name: "cargo".to_string(),
        binary: BinarySpecifier::Single("cargo".to_string()),
        list_cmd: "cargo install-list --installed 2>/dev/null || echo ''".to_string(),
        install_cmd: "cargo install {packages}".to_string(),
        remove_cmd: "cargo uninstall {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::Custom, // Use Rust fallback for now
        list_name_col: None,
        list_version_col: None,
        list_json_path: None,
        list_name_key: None,
        list_version_key: None,
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: None,
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: true, // Needs custom Rust implementation
    });

    // === Homebrew Backend ===
    backends.insert("brew".to_string(), BackendConfig {
        name: "brew".to_string(),
        binary: BinarySpecifier::Single("brew".to_string()),
        list_cmd: "brew list --versions".to_string(),
        install_cmd: "brew install {packages}".to_string(),
        remove_cmd: "brew uninstall {packages}".to_string(),
        query_cmd: None,
        list_format: crate::backends::config::OutputFormat::SplitWhitespace,
        list_name_col: Some(0),
        list_version_col: None, // Multiple versions possible
        list_json_path: None,
        list_name_key: None,
        list_version_key: None,
        list_regex: None,
        list_regex_name_group: None,
        list_regex_version_group: None,
        noconfirm_flag: Some("--force".to_string()),
        needs_sudo: false,
        preinstall_env: None,
        use_rust_fallback: false,
    });

    backends
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_backends() {
        let backends = get_builtin_backends();

        assert!(backends.contains_key("npm"));
        assert!(backends.contains_key("pip"));
        assert!(backends.contains_key("cargo"));
        assert!(backends.contains_key("brew"));
    }

    #[test]
    fn test_npm_config() {
        let backends = get_builtin_backends();
        let npm = &backends["npm"];

        assert_eq!(npm.name, "npm");
        assert!(!npm.needs_sudo);
        assert_eq!(npm.list_format, crate::backends::config::OutputFormat::Json);
    }

    #[test]
    fn test_pip_config() {
        let backends = get_builtin_backends();
        let pip = &backends["pip"];

        assert_eq!(pip.name, "pip");
        assert!(!pip.needs_sudo);
        assert_eq!(pip.list_format, crate::backends::config::OutputFormat::Json);
    }
}
