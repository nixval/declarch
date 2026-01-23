# User-Defined Backends - Implementation Plan

## 📋 Overview

Enable users to define custom package manager backends via KDL configuration files, making declarch extensible to any package manager wrapper or helper tool.

**Current State**: Backends hardcoded in Rust (`src/backends/registry.rs`)
**Target State**: Users can define backends in `~/.config/declarch/backends.kdl`

---

## 🎯 Goals

1. **Flexibility**: Support any package manager that follows common patterns
2. **Simplicity**: Easy KDL syntax for defining backends
3. **Validation**: Catch configuration errors early
4. **Documentation**: Comprehensive examples for popular backends

---

## 📐 Phase 1: Core KDL Parser (Foundation)

**Files**: `src/backends/user_parser.rs` (new), `src/backends/mod.rs` (update)

### Task: Parse Backend Definitions from KDL

```kdl
// ~/.config/declarch/backends.kdl

backend "nala" {
    binary "nala"
    list "nala list --installed" {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }
    install "nala install -y {packages}"
    remove "nala remove -y {packages}"
    noconfirm "-y"
}
```

**Implementation**:
```rust
// src/backends/user_parser.rs

use crate::backends::config::{BackendConfig, BinarySpecifier, OutputFormat};
use crate::error::{DeclarchError, Result};
use kdl::KdlDocument;
use std::path::Path;

pub fn load_user_backends(path: &Path) -> Result<Vec<BackendConfig>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let kdl_source = std::fs::read_to_string(path)
        .map_err(|e| DeclarchError::Other(format!("Failed to read backends.kdl: {}", e)))?;

    let doc = KdlDocument::parse(&kdl_source)
        .map_err(|e| DeclarchError::Other(format!("Failed to parse backends.kdl: {}", e)))?;

    let mut backends = Vec::new();

    for node in doc.nodes() {
        if node.name().value() == "backend" {
            let config = parse_backend_node(node)?;
            backends.push(config);
        }
    }

    Ok(backends)
}

fn parse_backend_node(node: &kdl::KdlNode) -> Result<BackendConfig> {
    // Extract backend name from argument
    let name = node.args().first()
        .and_then(|arg| arg.as_string())
        .ok_or_else(|| DeclarchError::Other("Backend name required".to_string()))?
        .to_string();

    let mut config = BackendConfig::default();
    config.name = name.clone();

    // Parse children
    for child in node.children() {
        match child.name().value() {
            "binary" => parse_binary(child, &mut config)?,
            "list" => parse_list_cmd(child, &mut config)?,
            "install" => parse_install_cmd(child, &mut config)?,
            "remove" => parse_remove_cmd(child, &mut config)?,
            "noconfirm" => parse_noconfirm(child, &mut config)?,
            "needs_sudo" => config.needs_sudo = parse_bool(child)?,
            "env" => parse_env(child, &mut config)?,
            _ => {
                return Err(DeclarchError::Other(format!(
                    "Unknown backend field: {}",
                    child.name().value()
                )));
            }
        }
    }

    // Validate required fields
    validate_backend_config(&config)?;

    Ok(config)
}

fn parse_binary(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    let values: Vec<_> = node.args().iter()
        .filter_map(|arg| arg.as_string())
        .collect();

    config.binary = match values.len() {
        0 => return Err(DeclarchError::Other("Binary requires at least one value".to_string())),
        1 => BinarySpecifier::Single(values[0].to_string()),
        _ => BinarySpecifier::Multiple(values.iter().map(|s| s.to_string()).collect()),
    };

    Ok(())
}

fn parse_list_cmd(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Extract command from argument
    let cmd = node.args().first()
        .and_then(|arg| arg.as_string())
        .ok_or_else(|| DeclarchError::Other("List command required".to_string()))?;

    config.list_cmd = cmd.to_string();

    // Parse output format from children
    for child in node.children() {
        match child.name().value() {
            "format" => {
                let format_str = child.args().first()
                    .and_then(|arg| arg.as_string())
                    .ok_or_else(|| DeclarchError::Other("Format value required".to_string()))?;

                config.list_format = match format_str {
                    "json" => OutputFormat::Json,
                    "whitespace" => OutputFormat::SplitWhitespace,
                    "tsv" => OutputFormat::TabSeparated,
                    "regex" => OutputFormat::Regex,
                    _ => return Err(DeclarchError::Other(format!("Unknown format: {}", format_str))),
                };
            }
            "json_path" => config.list_json_path = Some(child.args().first()
                .and_then(|arg| arg.as_string())
                .unwrap_or("").to_string()),
            "name_key" => config.list_name_key = child.args().first()
                .and_then(|arg| arg.as_string())
                .map(|s| s.to_string()),
            "version_key" => config.list_version_key = child.args().first()
                .and_then(|arg| arg.as_string())
                .map(|s| s.to_string()),
            "name_col" => config.list_name_col = child.args().first()
                .and_then(|arg| arg.as_i64())
                .map(|i| i as usize),
            "version_col" => config.list_version_col = child.args().first()
                .and_then(|arg| arg.as_i64())
                .map(|i| i as usize),
            "regex" => config.list_regex = child.args().first()
                .and_then(|arg| arg.as_string())
                .map(|s| s.to_string()),
            "name_group" => config.list_regex_name_group = child.args().first()
                .and_then(|arg| arg.as_i64())
                .map(|i| i as usize),
            "version_group" => config.list_regex_version_group = child.args().first()
                .and_then(|arg| arg.as_i64())
                .map(|i| i as usize),
            _ => {}
        }
    }

    Ok(())
}

fn parse_install_cmd(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.install_cmd = node.args().first()
        .and_then(|arg| arg.as_string())
        .ok_or_else(|| DeclarchError::Other("Install command required".to_string()))?
        .to_string();

    Ok(())
}

fn parse_remove_cmd(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.remove_cmd = node.args().first()
        .and_then(|arg| arg.as_string())
        .ok_or_else(|| DeclarchError::Other("Remove command required".to_string()))?
        .to_string();

    Ok(())
}

fn parse_noconfirm(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    config.noconfirm_flag = node.args().first()
        .and_then(|arg| arg.as_string())
        .map(|s| s.to_string());
    Ok(())
}

fn parse_bool(node: &kdl::KdlNode) -> Result<bool> {
    node.args().first()
        .and_then(|arg| arg.as_bool())
        .ok_or_else(|| DeclarchError::Other("Boolean value required".to_string()))
}

fn parse_env(node: &kdl::KdlNode, config: &mut BackendConfig) -> Result<()> {
    // Parse environment variables: env VAR="value" VAR2="value2"
    let mut env_map = std::collections::HashMap::new();

    for arg in node.args() {
        if let Some(s) = arg.as_string() {
            if let Some((key, value)) = s.split_once('=') {
                env_map.insert(key.to_string(), value.to_string());
            }
        }
    }

    if !env_map.is_empty() {
        config.preinstall_env = Some(env_map);
    }

    Ok(())
}

fn validate_backend_config(config: &BackendConfig) -> Result<()> {
    if config.list_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'list_cmd' cannot be empty".to_string()
        ));
    }

    if config.install_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'install_cmd' cannot be empty".to_string()
        ));
    }

    if config.remove_cmd.is_empty() {
        return Err(DeclarchError::Other(
            "Backend 'remove_cmd' cannot be empty".to_string()
        ));
    }

    // Validate format-specific requirements
    match config.list_format {
        OutputFormat::Json => {
            if config.list_name_key.is_none() {
                return Err(DeclarchError::Other(
                    "JSON format requires 'name_key' to be specified".to_string()
                ));
            }
        }
        OutputFormat::Regex => {
            if config.list_regex.is_none() {
                return Err(DeclarchError::Other(
                    "Regex format requires 'regex' to be specified".to_string()
                ));
            }
        }
        _ => {}
    }

    Ok(())
}
```

**Deliverables**:
- ✅ KDL parser for backend definitions
- ✅ Validation logic
- ✅ Error handling with helpful messages

**Testing**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_backend() {
        let kdl = r#"
            backend "test" {
                binary "test-manager"
                list "test list"
                install "test install {packages}"
                remove "test remove {packages}"
            }
        "#;

        let doc = KdlDocument::parse(kdl).unwrap();
        let node = doc.nodes().first().unwrap();
        let config = parse_backend_node(node).unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.list_cmd, "test list");
    }
}
```

---

## 📐 Phase 2: Backend Registry Enhancement

**File**: `src/backends/registry.rs` (update)

### Task: Merge User + Built-in Backends

```rust
use crate::backends::user_parser;
use crate::utils::paths;

pub fn load_all_backends() -> Result<HashMap<String, BackendConfig>> {
    let mut all_backends = get_builtin_backends();

    // Load user-defined backends
    let backends_path = paths::backend_config()?;

    if backends_path.exists() {
        let user_backends = user_parser::load_user_backends(&backends_path)?;

        // User backends can override built-ins
        for config in user_backends {
            all_backends.insert(config.name.clone(), config);
        }
    }

    Ok(all_backends)
}
```

**Helper function**:
```rust
// src/utils/paths.rs

pub fn backend_config() -> Result<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|| ".".to_string());
            format!("{}/.config", home)
        });

    Ok(PathBuf::from(config_dir).join("declarch").join("backends.kdl"))
}
```

---

## 📐 Phase 3: Placeholder Expansion System

**File**: `src/backends/placeholder.rs` (new)

### Task: Implement Smart Placeholder Replacement

**Supported placeholders**:
- `{packages}` → List of packages to install/remove
- `{package}` → Single package (for loop operations)
- `{format}` → Output format (json, text, etc.)
- `{env:VAR}` → Environment variable
- `{flags:noconfirm}` → Conditional noconfirm flag
- `{flags:sudo}` → Conditional sudo

```rust
use std::collections::HashMap;

pub fn expand_placeholders(
    template: &str,
    packages: &[String],
    flags: &PlaceholderFlags,
) -> String {
    let mut result = template.to_string();

    // Replace {packages}
    if result.contains("{packages}") {
        let packages_str = packages.join(" ");
        result = result.replace("{packages}", &packages_str);
    }

    // Replace conditional flags
    if flags.noconfirm {
        if let Some(noconfirm_flag) = &flags.noconfirm_value {
            result = result.replace("{flags:noconfirm}", noconfirm_flag);
        }
    } else {
        result = result.replace("{flags:noconfirm}", "");
    }

    // Expand environment variables
    result = expand_env_vars(&result);

    result
}

pub struct PlaceholderFlags {
    pub noconfirm: bool,
    pub noconfirm_value: Option<String>,
    pub sudo: bool,
    pub dry_run: bool,
    pub env: HashMap<String, String>,
}

fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Simple {env:VAR} expansion
    while let Some(start) = result.find("{env:") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 5..start + end];
            let full_placeholder = &result[start..start + end + 1];

            if let Ok(value) = std::env::var(var_name) {
                result = result.replace(full_placeholder, &value);
            } else {
                // Remove placeholder if env var doesn't exist
                result = result.replace(full_placeholder, "");
            }
        } else {
            break;
        }
    }

    result
}
```

---

## 📐 Phase 4: Advanced Features

### 4.1 Multiple List Commands with Fallback

```kdl
backend "dnf5" {
    binary ["dnf5", "dnf"]

    list_try [
        "dnf5 list installed --json",
        "dnf list installed --json"
    ] {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }

    install "dnf5 install -y {packages}"
    remove "dnf5 remove -y {packages}"
    needs_sudo true
}
```

### 4.2 Conditional Logic

```kdl
backend "apt" {
    binary "apt"

    list "apt list --installed" {
        format regex
        regex "([^/]+)\\s+([^\\s]+)\\s+(.+)"
        name_group 1
        version_group 2
    }

    install "apt install {flags:noconfirm} {packages}"

    noconfirm "--yes"
    needs_sudo true
}
```

### 4.3 Backend Inheritance/Templates

```kdl
// Define template
template "json_backend" {
    list_format json
    list_json_path ""
    list_name_key "name"
    list_version_key "version"
    noconfirm "-y"
}

// Use template
backend "poetry" extends json_backend {
    binary "poetry"
    list "poetry show --installed"
    install "poetry add {packages}"
    remove "poetry remove {packages}"
}
```

---

## 📐 Phase 5: Validation & Testing

### 5.1 Schema Validation

```rust
pub struct BackendSchema {
    required_fields: Vec<&'static str>,
    format_requirements: HashMap<OutputFormat, Vec<&'static str>>,
}

impl BackendSchema {
    pub fn validate(&self, config: &BackendConfig) -> Result<()> {
        // Check required fields
        for field in &self.required_fields {
            // Validation logic
        }

        // Check format-specific requirements
        match config.list_format {
            OutputFormat::Json => {
                // Validate JSON-specific fields
            }
            _ => {}
        }

        Ok(())
    }
}
```

### 5.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_user_backend_nala() {
        let kdl = r#"
            backend "nala" {
                binary "nala"
                list "nala list --installed" {
                    format json
                    json_path "packages"
                    name_key "name"
                    version_key "version"
                }
                install "nala install -y {packages}"
                remove "nala remove -y {packages}"
            }
        "#;

        // Parse and validate
    }

    #[test]
    fn test_backend_override_builtin() {
        // Test that user backends override built-ins
    }
}
```

---

## 📐 Phase 6: Documentation & Examples

**File**: `docs/User-Defined-Backends.md` (new)

Comprehensive documentation with real-world examples for:
- nala (Debian/Ubuntu)
- zypper (openSUSE)
- dnf5 (Fedora)
- poetry (Python)
- cargo (existing, as example)
- Custom wrappers

---

## 📐 Phase 7: Error Messages & UX

### Better Error Messages

```rust
pub fn format_backend_error(error: &DeclarchError) -> String {
    match error {
        DeclarchError::BackendNotFound { backend } => {
            format!(
                "Backend '{}' not found.\n\
                 Available: {}\n\
                 You can define custom backends in ~/.config/declarch/backends.kdl",
                backend,
                get_available_backends().join(", ")
            )
        }
        DeclarchError::InvalidBackendConfig { field, reason } => {
            format!(
                "Invalid backend configuration:\n\
                 Field '{}': {}\n\
                 See docs/User-Defined-Backends.md for examples",
                field, reason
            )
        }
        _ => error.to_string()
    }
}
```

---

## 📐 Phase 8: CLI Integration

### Command: `declarch backend list`

```bash
$ declarch backend list

Built-in backends:
  aur      - Arch User Repository
  flatpak  - Flatpak applications
  soar     - Static binary manager
  npm      - Node.js global packages
  pip      - Python packages
  cargo    - Rust crates
  brew     - Homebrew packages

User-defined backends:
  nala     - Debian/Ubuntu packages (from ~/.config/declarch/backends.kdl)
  zypper   - openSUSE packages (from ~/.config/declarch/backends.kdl)
```

### Command: `declarch backend validate`

```bash
$ declarch backend validate

✓ All backends validated successfully

User backends: 2
  ✓ nala
  ✓ zypper
```

---

## 📐 Phase 9: Performance & Caching

### Cache Binary Availability

```rust
use std::time::{Duration, Instant};

pub struct BackendCache {
    available_binaries: HashMap<String, Option<Instant>>,
    ttl: Duration,
}

impl BackendCache {
    pub fn is_available(&mut self, backend: &str) -> bool {
        if let Some(cached) = self.available_binaries.get(backend) {
            if let Ok(elapsed) = cached.elapsed() {
                if elapsed < self.ttl {
                    return true; // Still in cache
                }
            }
        }

        // Check availability
        let available = which::which(backend).is_ok();
        self.available_binaries.insert(
            backend.to_string(),
            Some(Instant::now())
        );

        available
    }
}
```

---

## 📐 Phase 10: Security Considerations

### Sanitize Backend Commands

```rust
pub fn sanitize_command(cmd: &str) -> Result<String> {
    // Prevent command injection
    if cmd.contains('|') || cmd.contains(';') || cmd.contains('&') {
        return Err(DeclarchError::Other(
            "Backend commands cannot contain shell operators |, ;, &".to_string()
        ));
    }

    // Validate placeholders
    // Allow only: {packages}, {package}, {env:VAR}, {flags:*}

    Ok(cmd.to_string())
}
```

---

## 🎯 Success Criteria

- ✅ Users can define backends in KDL
- ✅ User backends override built-ins
- ✅ Validation catches errors early
- ✅ Comprehensive documentation with examples
- ✅ Support for popular backends (nala, zypper, dnf5)
- ✅ Security (command sanitization)
- ✅ Performance (caching)
- ✅ Good error messages

---

## 📊 Timeline Estimate

| Phase | Complexity | Time |
|-------|------------|------|
| Phase 1: KDL Parser | Medium | 4-6 hours |
| Phase 2: Registry Merge | Low | 1-2 hours |
| Phase 3: Placeholders | Medium | 3-4 hours |
| Phase 4: Advanced | High | 8-10 hours |
| Phase 5: Validation | Medium | 2-3 hours |
| Phase 6: Documentation | Low | 2-3 hours |
| Phase 7: Error Messages | Low | 1-2 hours |
| Phase 8: CLI Commands | Medium | 3-4 hours |
| Phase 9: Performance | Low | 2-3 hours |
| Phase 10: Security | Medium | 2-3 hours |
| **Total** | | **28-43 hours** |

---

## 🚀 Next Steps

1. Create issue tracking for all phases
2. Start with Phase 1 (KDL Parser) - highest priority
3. Add tests alongside implementation
4. Document each phase as we go
5. Get user feedback early (after Phase 3)
