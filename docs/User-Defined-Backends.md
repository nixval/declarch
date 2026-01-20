# User-Defined Backends Guide

## 📚 Table of Contents

1. [Quick Start](#quick-start)
2. [Backend Definition Syntax](#backend-definition-syntax)
3. [Production-Ready Examples](#production-ready-examples)
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
    // Required: Binary name (can have multiple alternatives)
    binary "<command>" "<alternative-cmd>"

    // Required: List installed packages
    list "<command>" {
        format <format>  // json, whitespace, tsv, or regex
        // Format-specific options
    }

    // Required: Install packages
    install "<command>"

    // Required: Remove packages
    remove "<command>"

    // Optional: No-confirmation flag
    noconfirm "<flag>"

    // Optional: Requires sudo? (default: false)
    needs_sudo true

    // Optional: Environment variables
    env VAR="value" VAR2="value2"
}
```

### Placeholders

In `install` and `remove` commands:
- `{packages}` → Replaced with list of packages (e.g., `pkg1 pkg2 pkg3`)

---

## 💼 Production-Ready Examples

### Example 1: NALA (Debian/Ubuntu) ✅ TESTED

**What is Nala?**
- Modern CLI for APT (Debian/Ubuntu)
- Faster and prettier than apt
- JSON output support

**Installation:**
```bash
sudo apt install nala
```

**Configuration (`~/.config/declarch/backends.kdl`):**
```kdl
backend "nala" {
    binary "nala"

    list "nala list --installed --json" {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }

    install "nala install -y {packages}"
    remove "nala remove -y {packages}"

    noconfirm "-y"
    needs_sudo true
}
```

**Usage in `declarch.kdl`:**
```kdl
packages:nala {
    vim
    neovim
    git
    curl
}
```

**How it works:**
- `nala list --installed --json` outputs JSON with `packages` array
- Each package has `name` and `version` fields
- `-y` flag enables non-interactive mode

---

### Example 2: ZYPPER (openSUSE) ✅ TESTED

**What is Zypper?**
- Package manager for openSUSE
- Handles RPM packages
- Tab-separated output with `|` visual separators

**Installation:**
```bash
# Already installed on openSUSE
# Or: sudo zypper install zypper
```

**Configuration (`~/.config/declarch/backends.kdl`):**
```kdl
backend "zypper" {
    binary "zypper"

    list "zypper search --installed-only --type package --details" {
        format tsv
        name_col 2     // 3rd column: package name
        version_col 3  // 4th column: version
    }

    install "zypper install --no-confirm {packages}"
    remove "zypper remove --no-confirm {packages}"

    noconfirm "--no-confirm"
    needs_sudo true
}
```

**Usage in `declarch.kdl`:**
```kdl
packages:zypper {
    firefox
    python3
    gcc
    neovim
}
```

**How it works:**
- Zypper outputs tab-separated columns with visual `|` separators
- Column 0: Status (`i+` for installed)
- Column 1: Repository
- Column 2: Package name ← We use this
- Column 3: Version ← We use this
- Column 4: Architecture
- `--no-confirm` skips all prompts

---

### Example 3: DNF5 (Fedora) ✅ TESTED

**What is DNF5?**
- Next-gen DNF (Fedora package manager)
- JSON output support
- Faster than dnf
- Default in Fedora 39+

**Installation:**
```bash
# On Fedora 39+: Already installed
# On older versions: sudo dnf install dnf5
```

**Configuration (`~/.config/declarch/backends.kdl`):**
```kdl
backend "dnf5" {
    // Try dnf5 first, fallback to dnf
    binary ["dnf5", "dnf"]

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

**Usage in `declarch.kdl`:**
```kdl
packages:dnf5 {
    neovim
    podman
    toolbox
    ffmpeg
}
```

**How it works:**
- `dnf5 list installed --json` outputs JSON with `packages` array
- Each package has `name` and `version` fields
- Binary fallback: tries `dnf5` first, uses `dnf` if not available
- `-y` flag enables non-interactive mode

---

### Example 4: APT (Debian/Ubuntu) ✅ TESTED

**What is APT?**
- Standard package manager for Debian/Ubuntu
- Text-based output requiring regex parsing

**Configuration (`~/.config/declarch/backends.kdl`):**
```kdl
backend "apt" {
    binary "apt"

    list "apt list --installed" {
        format regex
        regex "^([^/]+)\\s+([^\\s]+)\\s+"
        name_group 1
        version_group 2
    }

    install "apt install --yes {packages}"
    remove "apt remove --yes {packages}"

    noconfirm "--yes"
    needs_sudo true
}
```

**Usage in `declarch.kdl`:**
```kdl
packages:apt {
    vim
    git
    curl
    htop
}
```

**How it works:**
- `apt list --installed` outputs: `vim/amd64 2:9.0.1000-0ubuntu1 amd64`
- Regex extracts:
  - Group 1: Package name (everything before `/`)
  - Group 2: Version (after name, before space)
- `--yes` skips confirmation prompts

---

### Example 5: PACMAN (Arch Linux Alternative) ✅ TESTED

**What is Pacman?**
- Standard package manager for Arch Linux
- Simple space-separated output

**Configuration (`~/.config/declarch/backends.kdl`):**
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

**Usage in `declarch.kdl`:**
```kdl
packages:pacman {
    neovim
    git
    fish
    fzf
}
```

**How it works:**
- `pacman -Q` outputs: `package-name version`
- Column 0: Package name
- Column 1: Version
- `--noconfirm` skips all prompts

---

## 📖 Output Format Reference

### 1. JSON Format

For package managers that output JSON:

```kdl
list "<cmd>" {
    format json
    json_path "packages"      // Path to packages array (optional)
    name_key "name"           // Key for package name
    version_key "version"     // Key for version
}
```

**Example output:**
```json
{
  "packages": [
    {"name": "vim", "version": "9.0.1000"},
    {"name": "git", "version": "2.43.0"}
  ]
}
```

**Supported backends:** nala, dnf5, npm, pip, pnpm, bun

---

### 2. TSV (Tab-Separated) Format

For tab-delimited output:

```kdl
list "<cmd>" {
    format tsv
    name_col 0   // First column (0-indexed)
    version_col 1  // Second column
}
```

**Example output:**
```
vim	9.0.1000
git	2.43.0
```

**Supported backends:** zypper, flatpak

---

### 3. Whitespace Format

For space-separated output:

```kdl
list "<cmd>" {
    format whitespace
    name_col 0
    version_col 1
}
```

**Example output:**
```
vim 9.0.1000
git 2.43.0
```

**Supported backends:** pacman, cargo, brew

---

### 4. Regex Format

For complex output patterns:

```kdl
list "<cmd>" {
    format regex
    regex "([^/]+)\\s+([^\\s]+)\\s+"
    name_group 1    // First capture group
    version_group 2  // Second capture group
}
```

**Example output:**
```
vim/amd64 9.0.1000-0ubuntu1 amd64
git/amd64 2.43.0-0ubuntu1 amd64
```

**Regex breakdown:**
- `([^/]+)` → Capture everything before `/` (package name)
- `\\s+` → One or more spaces
- `([^\\s]+)` → Capture non-space characters (version)

**Supported backends:** apt

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

### Advanced Template: Custom Backend

For advanced users wanting to add a completely custom package manager:

```kdl
// ~/.config/declarch/backends.kdl

backend "my-custom-manager" {
    // Multiple binary options for compatibility
    binary ["my-manager", "my-manager-v2", "/usr/local/bin/my-manager"]

    // List command with JSON output
    list "my-manager list --all --json" {
        format json

        // Navigate to nested array: results[].packages
        json_path "results.packages"

        // Custom field names
        name_key "pkg_id"
        version_key "installed_ver"

        // Optional: filter by status
        // regex "status:installed"
    }

    // Install command
    install "my-manager install --batch {packages}"

    // Remove command
    remove "my-manager uninstall --batch --purge {packages}"

    // Auto-confirmation
    noconfirm "--batch"

    // Requires root privileges
    needs_sudo true

    // Environment variables
    env MY_MANAGER_CACHE="/tmp/my-manager-cache"
    env MY_MANAGER_LOG_LEVEL="info"
    env MY_MANAGER_NO_INTERACTIVE="1"
}
```

**Usage in `declarch.kdl`:**
```kdl
packages:my-custom-manager {
    my-package
    another-package
}
```

**Testing your backend:**
```bash
# Check if backend is loaded
declarch check --backend my-custom-manager --verbose

# Test list command
my-manager list --all --json | jq '.results.packages[0]'

# Dry-run sync
declarch sync --dry-run --backend my-custom-manager
```

---

## 🐛 Troubleshooting

### Backend Not Found

**Error:**
```
Error: Backend 'nala' not found
```

**Solution:**
1. Check backend is defined in `~/.config/declarch/backends.kdl`
2. Run `declarch check --backend nala` to see available backends
3. Validate KDL syntax:

```bash
# Check syntax
cat ~/.config/declarch/backends.kdl

# Look for common errors:
# - Missing quotes around strings
# - Unmatched braces
# - Invalid format names
```

---

### Binary Not Available

**Error:**
```
Error: Binary 'nala' not found in PATH
```

**Solution:**
1. Install the package manager:
   ```bash
   sudo apt install nala  # Debian/Ubuntu
   sudo zypper install nala  # openSUSE
   sudo dnf install nala  # Fedora
   ```
2. Check binary exists: `which nala`
3. Verify binary name in backend config

---

### List Command Failed

**Error:**
```
Error: Failed to query nala package database
```

**Solution:**
1. Test the list command manually:
   ```bash
   nala list --installed --json | head -20
   ```

2. Check output format matches config:
   ```bash
   # For JSON formats
   nala list --installed --json | jq '.packages[0]'

   # For TSV formats
   zypper search --installed-only | head -5

   # For whitespace formats
   pacman -Q | head -5
   ```

3. Verify JSON path or regex pattern:
   ```bash
   # Test JSON path
   nala list --installed --json | jq '.packages[].name'

   # Test regex (requires ripgrep or similar)
   zypper search --installed-only | grep -E "^i\+.*"
   ```

---

### Parsing Errors

**Symptom:** Packages not appearing in `declarch check --verbose`

**Debug steps:**
1. Enable verbose mode:
   ```bash
   declarch check --backend <name> --verbose
   ```

2. Check raw output:
   ```bash
   # Replace with your list command
   <list-cmd> > /tmp/output.txt
   cat /tmp/output.txt
   ```

3. Validate format:
   ```bash
   # JSON: Check structure
   cat /tmp/output.txt | jq '.'

   # TSV: Check tabs
   cat -A /tmp/output.txt | head -3

   # Whitespace: Check columns
   cat /tmp/output.txt | head -3 | awk '{print NF}'
   ```

4. Common issues:
   - **JSON:** Wrong `json_path` or `name_key`
   - **TSV:** Wrong column numbers (0-indexed!)
   - **Whitespace:** Columns don't match after splitting
   - **Regex:** Pattern doesn't match output

---

### Validation Errors

**Error:**
```
Error: Invalid backend configuration:
 Field 'name_key': Required for JSON format
```

**Solution:**
Add missing required fields based on output format:

- **JSON:** `name_key`, `version_key`
- **Regex:** `regex`, `name_group`, `version_group`
- **Whitespace/TSV:** `name_col`, `version_col`

---

## 🎯 Best Practices

1. **Test commands manually** before defining in backend
2. **Start with simple format** (whitespace), then optimize
3. **Use JSON format** when available (most reliable)
4. **Add multiple binary alternatives** for compatibility
5. **Document custom backends** for your team/org
6. **Always test with `--dry-run`** before actual sync:
   ```bash
   declarch sync --dry-run --backend <name>
   ```

---

## 🧪 Testing Your Backend

### Step 1: Verify Backend Configuration

```bash
# Check if backend is recognized
declarch check --backend <name>

# Should see:
# ✓ Syntax & Imports: OK
# ℹ Filtering by backend: <name>
# Unique Packages: <count>
```

### Step 2: Test List Command Manually

```bash
# Run the list command from your backend config
<list-command> | head -5

# Examples:
nala list --installed --json | head -5
zypper search --installed-only | head -5
dnf5 list installed --json | head -5
```

### Step 3: Parse Test

```bash
# Save output to file
<list-command> > /tmp/test-output.txt

# Check format
# For JSON:
cat /tmp/test-output.txt | jq '.'

# For TSV/Whitespace:
cat /tmp/test-output.txt | head -3
```

### Step 4: Dry Run Sync

```bash
# Test without making changes
declarch sync --dry-run --backend <name> --verbose
```

### Step 5: Actual Sync

```bash
# After verifying dry-run output
declarch sync --backend <name>
```

---

## 📚 Additional Resources

- [Backend System Documentation](Backend-System.md) - Internal architecture
- [Generic Manager Guide](Backend-System.md#generic-manager) - How backends work
- [KDL Syntax Reference](KDL-Syntax-Reference.md) - KDL configuration format
- [Testing Guide](../README.md#testing) - Testing new backends

---

## 🤝 Contributing

Have a backend that others might use?

1. **Test it thoroughly** on multiple systems
2. **Add comprehensive example** to this document
3. **Consider contributing** to built-in backends
4. **Submit PR** with:
   - Backend configuration
   - Test cases
   - Documentation
   - Example usage

Example PR structure:
```rust
// src/backends/registry.rs

backends.insert("your_backend".to_string(), BackendConfig {
    name: "your_backend".to_string(),
    binary: BinarySpecifier::Single("your-cmd".to_string()),
    list_cmd: "your-cmd list --json".to_string(),
    list_format: OutputFormat::Json,
    list_json_path: Some("packages".to_string()),
    list_name_key: Some("name".to_string()),
    list_version_key: Some("version".to_string()),
    install_cmd: "your-cmd install -y {packages}".to_string(),
    remove_cmd: "your-cmd remove -y {packages}".to_string(),
    noconfirm_flag: Some("-y".to_string()),
    needs_sudo: false,
    ..Default::default()
});
```

---

## 🔗 Quick Reference

### Supported Distros & Backends

| Distro | Backend | Format |
|--------|---------|--------|
| Debian/Ubuntu | `nala` | JSON |
| Debian/Ubuntu | `apt` | Regex |
| openSUSE | `zypper` | TSV |
| Fedora | `dnf5` | JSON |
| Arch | `pacman` | Whitespace |
| Any | `flatpak` | TSV |
| Any | `npm` | JSON |
| Any | `pip` | JSON |
| Any | `cargo` | Whitespace |
| macOS/Linux | `brew` | Whitespace |

### Format Selection Guide

| Output Type | Best Format | Example |
|-------------|-------------|---------|
| JSON | `json` | `{"name": "pkg", "version": "1.0"}` |
| Tab-separated | `tsv` | `pkg\t1.0` |
| Space-separated | `whitespace` | `pkg 1.0` |
| Complex patterns | `regex` | `pkg/1.0 (installed)` |

---

## ✅ Checklist for New Backends

- [ ] Backend defined in `~/.config/declarch/backends.kdl`
- [ ] Binary exists in PATH
- [ ] List command tested manually
- [ ] Output format verified
- [ ] Columns/Keys match output
- [ ] Install/remove commands tested
- [ ] Noconfirm flag works
- [ ] Sudo requirement set correctly
- [ ] `declarch check --backend <name>` works
- [ ] `declarch sync --dry-run --backend <name>` tested
- [ ] Documentation updated
- [ ] Examples provided

---

**Last Updated:** 2025-01-20
**Tested On:**
- NALA: Debian/Ubuntu format (JSON)
- Zypper: openSUSE format (TSV)
- DNF5: Fedora 39+ format (JSON)
- APT: Debian/Ubuntu format (Regex)
- Pacman: Arch Linux format (Whitespace)
