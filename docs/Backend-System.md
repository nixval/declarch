# Generic Backend System

Declarch now supports a **generic backend system** that makes it easy to add new package managers without writing Rust code.

## Overview

The generic backend system allows you to:
- Add new package managers through configuration
- Use npm, pip, cargo, brew, yarn, pnpm, bun out of the box
- Define custom backends without recompiling declarch
- Keep existing Rust-optimized implementations for AUR, Flatpak, Soar

## Supported Backends

### Built-in Backends (via Configuration)

| Backend | Description | Binary | Example Usage |
|---------|-------------|--------|---------------|
| **npm** | Node.js global packages | `npm` | `packages:npm { typescript }` |
| **yarn** | Yarn global packages | `yarn` | `packages:yarn { prettier }` |
| **pnpm** | pnpm global packages | `pnpm` | `packages:pnpm { vite }` |
| **bun** | Bun global packages | `bun` | `packages:bun { bun }` |
| **pip** | Python packages | `pip3`, `pip` | `packages:pip { ruff }` |
| **cargo** | Rust crates | `cargo` | `packages:cargo { ripgrep }` |
| **brew** | Homebrew packages | `brew` | `packages:brew { node }` |

### Rust-Optimized Backends

| Backend | Description |
|---------|-------------|
| **AUR** | Arch User Repository (Rust implementation) |
| **Flatpak** | Flatpak applications (Rust implementation) |
| **Soar** | Static binaries (Rust implementation) |

## Usage

### Basic Example

```kdl
// ~/.config/declarch/declarch.kdl

meta {
    description "My development environment"
}

// Node.js packages
packages:npm {
    typescript
    prettier
    eslint
}

// Python packages
packages:pip {
    ruff
    black
    pytest
}

// Rust tools
packages:cargo {
    ripgrep
    fd-find
    eza
}

// Homebrew (macOS/Linux)
packages:brew {
    node
    python@3.12
}
```

### Applying Changes

```bash
# Preview changes
declarch sync --dry-run

# Apply changes
declarch sync
```

## Backend Configuration

Each backend is defined with:

1. **Binary**: Command to run (or alternatives to try)
2. **List Command**: How to list installed packages
3. **Install Command**: How to install packages
4. **Remove Command**: How to remove packages
5. **Output Format**: How to parse command output (JSON, whitespace, etc.)

### Example: npm Configuration

```rust
BackendConfig {
    name: "npm".to_string(),
    binary: BinarySpecifier::Single("npm".to_string()),
    list_cmd: "npm list -g --depth=0 --json 2>/dev/null".to_string(),
    install_cmd: "npm install -g {packages}".to_string(),
    remove_cmd: "npm uninstall -g {packages}".to_string(),
    list_format: OutputFormat::Json,
    list_json_path: Some("dependencies".to_string()),
    list_name_key: Some("name".to_string()),
    list_version_key: Some("version".to_string()),
    needs_sudo: false,
    ..Default::default()
}
```

## Output Formats

The generic backend supports multiple output formats:

### 1. SplitWhitespace (space-separated)
```
pacman 6.0.2
systemd 255.1
```
```kdl
list-format "split_whitespace"
list-name-col 0
list-version-col 1
```

### 2. TabSeparated (tab-separated)
```
com.spotify.Client	1.2.3
org.mozilla.firefox	120.0
```
```kdl
list-format "tab_separated"
list-name-col 0
list-version-col 1
```

### 3. Json (JSON output)
```json
{
  "dependencies": {
    "npm": {"name": "npm", "version": "11.6.2"}
  }
}
```
```kdl
list-format "json"
list-json-path "dependencies"
list-name-key "name"
list-version-key "version"
```

### 4. Regex (custom pattern)
```kdl
list-format "regex"
list-regex "package-(\\w+)-([\\d.]+)"
list-regex-name-group 1
list-regex-version-group 2
```

## Architecture

```
┌─────────────────────────────────────────────┐
│              User Configuration                │
│  packages:npm { typescript }               │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│            Backend Registry                   │
│  - Built-in configs (npm, pip, cargo...)    │
│  - Rust implementations (AUR, Flatpak...)    │
└─────────────────┬───────────────────────────┘
                  │
      ┌───────────┴───────────┐
      ▼                       ▼
┌───────────┐         ┌──────────────┐
│   Rust    │         │   Generic    │
│ Managers  │         │   Manager    │
│ (Optimized)│       │ (Config-Driven)│
└───────────┘         └──────────────┘
```

## Future: User-Defined Backends

In the future, users will be able to define custom backends in `~/.config/declarch/backends.kdl`:

```kdl
// ~/.config/declarch/backends.kdl

backends:my-custom-tool {
    binary "mytool"
    list-cmd "mytool list --json"
    install-cmd "mytool install {packages}"
    remove-cmd "mytool remove {packages}"

    list-format "json"
    list-json-path ""
    list-name-key "name"
    list-version-key "version"

    needs-sudo false
}
```

## Adding New Backends

To add a new built-in backend:

1. **Add to Backend enum** (`src/core/types.rs`):
```rust
pub enum Backend {
    // ... existing backends
    MyTool,  // New backend
}
```

2. **Add configuration** (`src/backends/registry.rs`):
```rust
backends.insert("mytool".to_string(), BackendConfig {
    name: "mytool".to_string(),
    binary: BinarySpecifier::Single("mytool".to_string()),
    list_cmd: "mytool list --json".to_string(),
    install_cmd: "mytool install {packages}".to_string(),
    remove_cmd: "mytool remove {packages}".to_string(),
    list_format: OutputFormat::Json,
    // ... more config
    ..Default::default()
});
```

3. **Register** in `register_defaults()`:
```rust
self.register(Backend::MyTool, |config, noconfirm| {
    Ok(Box::new(GenericManager::from_config(
        mytool_config.clone(),
        Backend::MyTool,
        noconfirm,
    )))
});
```

## Benefits

1. **No Rust Code Required**: Add npm, pip, cargo without writing Rust
2. **Easy Maintenance**: Update commands in one place
3. **Fast Development**: Test new backends quickly
4. **Backward Compatible**: Existing AUR, Flatpak, Soar still work
5. **Extensible**: Users can add custom backends in the future

## Testing

All backends are tested automatically:

```bash
# Run all tests
cargo test

# Run only backend tests
cargo test backends

# Run integration tests
cargo test --lib backends::integration_tests
```

## Contributing

To add a new backend:

1. Check if the package manager supports:
   - Listing installed packages
   - Installing packages
   - Removing packages

2. Determine output format (JSON, whitespace, TSV, etc.)

3. Add configuration following the pattern above

4. Add tests in `src/backends/integration_tests.rs`

5. Submit pull request!

## See Also

- [KDL Syntax Reference](KDL-Syntax-Reference.md)
- [Examples](Examples.md)
- [Main README](../README.md)
