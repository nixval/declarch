# User-Defined Backends Guide

## 📚 Table of Contents

1. [Quick Start](#quick-start)
2. [Backend Definition Syntax](#backend-definition-syntax)
3. [Complete Examples](#complete-examples)
4. [Output Format Reference](#output-format-reference)
5. [Advanced Features](#advanced-features)
6. [Troubleshooting](#troubleshooting)

---

## 🚀 Quick Start

### What Are User-Defined Backends?

User-defined backends allow you to extend declarch to support **any** package manager, wrapper, or helper tool by defining them in a simple KDL configuration file.

**Why use this?**
- Support distros not covered by built-in backends (Debian, openSUSE, Fedora, etc.)
- Add support for language-specific package managers (poetry, composer, go install, etc.)
- Integrate custom internal tools
- Test new package managers before contributing to declarch

---

## 📝 Backend Definition Syntax

### Location

Create/edit: `~/.config/declarch/backends.kdl`

### Basic Structure

```kdl
backend "<backend-name>" {
    // Required: Binary name
    binary "<command>"

    // Required: List installed packages
    list "<command>" {
        // Output format (json, whitespace, tsv, regex)
        format <format>
        // Format-specific options (see below)
    }

    // Required: Install packages
    install "<command>"

    // Required: Remove packages
    remove "<command>"

    // Optional: No-confirmation flag
    noconfirm "<flag>"

    // Optional: Requires sudo?
    needs_sudo true

    // Optional: Environment variables
    env VAR="value" VAR2="value2"
}
```

### Placeholders

In `install` and `remove` commands:
- `{packages}` → Replaced with list of packages (e.g., `pkg1 pkg2 pkg3`)
- `{flags:noconfirm}` → Conditional noconfirm flag

---

## 💡 Complete Examples

### Example 1: NALA (Debian/Ubuntu)

**What is Nala?**
- Modern CLI for APT (Debian/Ubuntu)
- Faster and prettier than apt
- JSON output support

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

**Usage in declarch.kdl**:
```kdl
packages:nala {
    vim
    neovim
    git
}
```

---

### Example 2: ZYPPER (openSUSE)

**What is Zypper?**
- Package manager for openSUSE
- Handles RPM packages
- Text-based output

```kdl
backend "zypper" {
    binary "zypper"

    list "zypper search --installed-only --type package" {
        format whitespace
        name_col 2
        version_col 4
    }

    install "zypper install --no-confirm {packages}"
    remove "zypper remove --no-confirm {packages}"

    noconfirm "--no-confirm"
    needs_sudo true
}
```

**Usage in declarch.kdl**:
```kdl
packages:zypper {
    firefox
    python3
    gcc
}
```

---

### Example 3: DNF5 (Fedora)

**What is DNF5?**
- Next-gen DNF (Fedora package manager)
- JSON output support
- Faster than dnf

```kdl
backend "dnf5" {
    binary ["dnf5", "dnf"]  // Try dnf5 first, fallback to dnf

    list "dnf5 list installed --json" {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }

    install "dnf5 install -y {packages}"
    remove "dnf5 remove -y {packages}"

    noconfirm "-y"
    needs_sudo true
}
```

**Usage in declarch.kdl**:
```kdl
packages:dnf5 {
    neovim
    podman
    toolbox
}
```

---

### Example 4: POETRY (Python)

**What is Poetry?**
- Python dependency and project manager
- Installs to project-local virtual environments

```kdl
backend "poetry" {
    binary "poetry"

    list "poetry show --installed" {
        format tsv
        name_col 0
        version_col 1
    }

    install "poetry add {packages}"
    remove "poetry remove {packages}"

    noconfirm "--no-interaction"
}
```

**Usage in declarch.kdl**:
```kdl
packages:poetry {
    requests
    pytest
    black
}
```

---

### Example 5: APT (Debian/Ubuntu - Standard)

```kdl
backend "apt" {
    binary "apt"

    list "apt list --installed" {
        format regex
        regex "([^/]+)\\s+([^\\s]+)\\s+(.+)"
        name_group 1
        version_group 2
    }

    install "apt install --yes {packages}"
    remove "apt remove --yes {packages}"

    noconfirm "--yes"
    needs_sudo true
}
```

---

### Example 6: PACMAN (Arch Linux - Alternative)

```kdl
backend "pacman" {
    binary "pacman"

    list "pacman -Q" {
        format whitespace
        name_col 0
        version_col 1
    }

    install "pacman -S --noconfirm {packages}"
    remove "pacman -R --noconfirm {packages}"

    noconfirm "--noconfirm"
    needs_sudo true
}
```

---

### Example 7: FLATPAK (User Override)

Override built-in backend with custom options:

```kdl
backend "flatpak" {
    binary "flatpak"

    list "flatpak list --columns=application,name,version" {
        format tsv
        name_col 0
        version_col 2
    }

    install "flatpak install -y {packages}"
    remove "flatpak uninstall -y {packages}"

    noconfirm "-y"
}
```

---

### Example 8: CUSTOM WRAPPER

```kdl
backend "my-wrapper" {
    binary "my-wrapper"

    list "my-wrapper list --json" {
        format json
        json_path "results"
        name_key "package_name"
        version_key "installed_version"
    }

    install "my-wrapper install --batch {packages}"
    remove "my-wrapper uninstall --batch {packages}"

    noconfirm "--batch"

    env MY_WRAPPER_CACHE="/tmp/my-wrapper-cache"
    env MY_WRAPPER_LOG_LEVEL="info"
}
```

---

## 📖 Output Format Reference

### 1. JSON Format

For package managers that output JSON:

```kdl
list "npm list -g --depth=0 --json" {
    format json
    json_path "dependencies"      // Path to packages array (optional)
    name_key "name"                // Key for package name
    version_key "version"         // Key for version
}
```

**Example output**:
```json
{
  "dependencies": {
    "npm": {"name": "npm", "version": "10.0.0"},
    "pnpm": {"name": "pnpm", "version": "8.0.0"}
  }
}
```

**For nested arrays**:
```kdl
list "cmd list" {
    format json
    json_path "results.packages"  // Navigate: results.packages
    name_key "id"
    version_key "ver"
}
```

---

### 2. Whitespace Format

For space-separated output:

```kdl
list "pacman -Q" {
    format whitespace
    name_col 0      // First column (0-indexed)
    version_col 1   // Second column
}
```

**Example output**:
```
neovim 0.9.1-1
python 3.11.5-1
```

---

### 3. TSV (Tab-Separated) Format

For tab-delimited output:

```kdl
list "flatpak list --columns=app,name,version" {
    format tsv
    name_col 0
    version_col 2
}
```

**Example output**:
```
org.gimp.GIMP    GIMP    2.10.36-1
org.mozilla.firefox    Firefox    120.0
```

---

### 4. Regex Format

For complex output patterns:

```kdl
list "apt list --installed" {
    format regex
    regex "([^/]+)\\s+([^\\s]+)\\s+(.+)"
    name_group 1        // First capture group
    version_group 2     // Second capture group
}
```

**Example output**:
```
vim/amd64 2:9.0.1000-0ubuntu1 amd64
```

**Regex breakdown**:
- `([^/]+)` → Capture everything before `/` (package name)
- `\\s+` → One or more spaces
- `([^\\s]+)` → Capture non-space characters (version)
- `\\s+` → One or more spaces
- `(.+)` → Rest of line (architecture)

---

## 🔧 Advanced Features

### Multiple Binary Alternatives

Try multiple binaries, use first available:

```kdl
backend "dnf5" {
    binary ["dnf5", "dnf", "yum"]  // Try in order

    // ...
}
```

---

### Environment Variables

Set environment variables for commands:

```kdl
backend "custom" {
    binary "custom"

    install "custom install {packages}"

    env MY_VAR="value"
    env PATH="/custom/path:$PATH"
}
```

---

### Sudo Requirement

```kdl
backend "system" {
    binary "apt"

    install "apt install -y {packages}"
    remove "apt remove -y {packages}"

    needs_sudo true  // Will prepend sudo to commands
}
```

---

### No-Confirmation Flag

```kdl
backend "example" {
    binary "example"

    install "example install {packages}"

    noconfirm "--yes"   // or "-y", "--batch", etc.
}
```

---

## 🐛 Troubleshooting

### Backend Not Found

**Error**:
```
Error: Backend 'nala' not found
```

**Solution**:
1. Check backend is defined in `~/.config/declarch/backends.kdl`
2. Run `declarch backend list` to see available backends
3. Validate syntax: `declarch backend validate`

---

### Binary Not Available

**Error**:
```
Error: Binary 'nala' not found in PATH
```

**Solution**:
1. Install the package manager: `sudo apt install nala`
2. Check binary exists: `which nala`
3. Verify binary name in backend config

---

### List Command Failed

**Error**:
```
Error: Failed to query nala package database
```

**Solution**:
1. Test the list command manually: `nala list --installed`
2. Check output format matches config
3. Verify JSON path or regex pattern

---

### Validation Errors

**Error**:
```
Error: Invalid backend configuration:
 Field 'name_key': Required for JSON format
```

**Solution**:
Add missing required fields based on output format:
- JSON: `name_key`, `version_key`
- Regex: `regex`, `name_group`, `version_group`
- Whitespace/TSV: `name_col`, `version_col`

---

### Debug Tips

Enable verbose mode:
```bash
declarch -v check
declarch -v sync
```

Test backend manually:
```bash
# Test list command
nala list --installed | head -20

# Test install
nala install -y test-package

# Check format
nala list --installed | jq '.packages[0]'
```

---

## 🎯 Best Practices

1. **Test commands manually** before defining in backend
2. **Start with simple format** (whitespace), then optimize
3. **Use JSON format** when available (most reliable)
4. **Add multiple binary alternatives** for compatibility
5. **Document custom backends** for your team/org
6. **Validate backend config**: `declarch backend validate`

---

## 📚 Additional Resources

- [Backend System Documentation](Backend-System.md) - Internal architecture
- [Generic Manager Guide](Backend-System.md#generic-manager) - How backends work
- [Testing Guide](../TESTING.md) - Testing new backends
- [Implementation Plan](User-Defined-Backends-Plan.md) - Development roadmap

---

## 🤝 Contributing

Have a backend that others might use?

1. Test it thoroughly
2. Add example to this document
3. Consider contributing to built-in backends
4. Submit PR with:

```rust
// src/backends/registry.rs

backends.insert("your_backend".to_string(), BackendConfig {
    name: "your_backend".to_string(),
    binary: BinarySpecifier::Single("your-cmd".to_string()),
    list_cmd: "...".to_string(),
    // ...
});
```
