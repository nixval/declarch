# Custom Backends

Extend declarch with custom package manager backends using the v0.6+ KDL-based configuration format.

## What Are Custom Backends?

Custom backends allow you to define support for package managers that aren't built into declarch. This is useful for:

- Distro-specific package managers (dnf, nala, apt, etc.)
- Language package managers not included (go, composer, etc.)
- Internal/private package managers
- Experimental or custom package management systems

## Backend Configuration Files

In v0.6+, backends are defined in separate KDL files stored in:

```
~/.config/declarch/backends/
```

Each backend has its own file named `<backend-name>.kdl`.

## Quick Start

### Create a Backend with `init --backend`

The easiest way to create a new backend:

```bash
# Create a new backend (e.g., cargo for Rust packages)
declarch init --backend cargo

# This creates ~/.config/declarch/backends/cargo.kdl
# Edit it to customize the commands
```

### Backend File Structure

A backend file looks like this:

```kdl
// cargo - Rust package manager
backend "cargo" {
    meta {
        title "Cargo"
        description "Rust package manager"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "cargo"
        homepage "https://doc.rust-lang.org/cargo/"
        license "MIT"
        created "2026-02-07"
        platforms "linux"
        requires "cargo"
    }
    
    // The binary to use (can specify multiple alternatives)
    binary "cargo"
    
    // Command to list installed packages
    list "cargo install --list" {
        format whitespace
        name_col 0
        version_col 1
    }
    
    // Install command - {packages} will be replaced with package names
    install "cargo install {packages}"
    
    // Remove command
    remove "cargo uninstall {packages}"
    
    // Search command (optional)
    search "cargo search {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
    
    // Auto-confirmation flag (optional)
    noconfirm "-y"
    
    // Whether this backend requires sudo (optional)
    needs_sudo false
    
    // Fallback backend if binary not found (optional, v0.6+)
    // fallback "apt"
}
```

## Required Fields

| Field | Description | Example |
|-------|-------------|---------|
| `backend` | Backend name in quotes | `"cargo"` |
| `binary` | Binary name or alternatives | `"cargo"` or `"pip3" "pip"` |
| `list` | List command with parser config | `"cargo install --list" { format whitespace }` |
| `install` | Install command template | `"cargo install {packages}"` |
| `remove` | Remove command template | `"cargo uninstall {packages}"` |

## Command Placeholders

Commands support the following placeholder:

| Placeholder | Replaced With |
|-------------|---------------|
| `{packages}` | Space-separated package names |
| `{query}` | Search query (for search command) |

### Example with Placeholders

```kdl
install "cargo install {packages}"
remove "cargo uninstall {packages}"
search "cargo search {query}"
```

## List Output Formats

The list command supports multiple parsing formats:

### whitespace (default)
Parse whitespace-separated columns:

```kdl
list "pip3 list" {
    format whitespace
    name_col 0      // First column is package name
    version_col 1   // Second column is version (optional)
}
```

### tsv
Parse tab-separated values:

```kdl
list "flatpak list --app --columns=application,version" {
    format tsv
    name_col 0
    version_col 1
}
```

### json
Parse JSON output:

```kdl
list "dnf5 list --installed --json" {
    format json
    json_path "packages"    // Path to array in JSON
    name_key "name"         // Key for package name
    version_key "version"   // Key for version
}
```

### regex
Parse with regular expression:

```kdl
list "custom-pkg list" {
    format regex
    pattern "^(\S+)\s+(\S+)"
    name_group 1
    version_group 2
}
```

## Examples

### NPM (Node.js)

```kdl
// ~/.config/declarch/backends/npm.kdl
backend "npm" {
    meta {
        title "NPM"
        description "Node.js package manager"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "nodejs" "npm"
        homepage "https://www.npmjs.com"
        created "2026-02-07"
        requires "npm"
    }
    
    binary "npm"
    
    list "npm list -g --depth=0" {
        format regex
        pattern "(\S+)@(\S+)"
        name_group 1
        version_group 2
    }
    
    install "npm install -g {packages}"
    remove "npm uninstall -g {packages}"
    
    search "npm search {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
}
```

### pip (Python)

```kdl
// ~/.config/declarch/backends/pip.kdl
backend "pip" {
    meta {
        title "pip"
        description "Python package installer"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "python" "pip"
        homepage "https://pip.pypa.io"
        created "2026-02-07"
        requires "pip3"
    }
    
    binary "pip3" "pip"
    
    list "pip3 list" {
        format whitespace
        name_col 0
        version_col 1
        skip_header 2  // Skip "Package Version" header lines
    }
    
    install "pip3 install {packages}"
    remove "pip3 uninstall {packages}"
    
    noconfirm "-y"
}
```

### Nala (Debian/Ubuntu)

```kdl
// ~/.config/declarch/backends/nala.kdl
backend "nala" {
    meta {
        title "Nala"
        description "APT frontend with better formatting"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "debian" "ubuntu" "apt"
        created "2026-02-07"
        requires "nala"
    }
    
    binary "nala"
    fallback "apt"
    
    list "nala list --installed" {
        format whitespace
        name_col 0
        version_col 1
    }
    
    install "nala install {packages}"
    remove "nala remove {packages}"
    
    noconfirm "-y"
    needs_sudo true
}
```

### DNF5 (Fedora)

```kdl
// ~/.config/declarch/backends/dnf5.kdl
backend "dnf5" {
    meta {
        title "DNF5"
        description "Modern Fedora package manager"
        version "1.0.0"
        author "declarch-user"
        tags "package-manager" "fedora" "rpm"
        created "2026-02-07"
        requires "dnf5"
    }
    
    binary "dnf5"
    
    list "dnf5 list --installed --json" {
        format json
        json_path ""
        name_key "name"
        version_key "version"
    }
    
    install "dnf5 install {packages}"
    remove "dnf5 remove {packages}"
    
    noconfirm "-y"
    needs_sudo true
}
```

## Advanced Features

### Multiple Binary Alternatives

Specify multiple binaries to try (first available wins):

```kdl
binary "pip3" "pip" "python3 -m pip"
```

### Fallback Backend

If the primary binary isn't found, use another backend:

```kdl
backend "paru" {
    binary "paru"
    fallback "yay"  // Try yay if paru not found
    
    // ... rest of config
}
```

### Environment Variables

Set environment variables for this backend:

```kdl
backend "custom" {
    binary "custom-pkg"
    
    env {
        "CUSTOM_HOME=/opt/custom"
        "CUSTOM_CACHE=/var/cache/custom"
    }
    
    // ... commands
}
```

## Testing Custom Backends

### 1. Validate Configuration

```bash
declarch check validate
```

### 2. Test List Command

```bash
# Check if backend is recognized
declarch info --backend cargo

# Should show backend info or "Backend 'cargo' not found" if invalid
```

### 3. Dry Run

Add to a module and test:

```kdl
// modules/dev.kdl
pkg {
    cargo {
        ripgrep
        fd-find
        bat
    }
}
```

```bash
declarch sync --dry-run
```

## Troubleshooting

### Backend Not Recognized

**Problem:** Backend doesn't appear in `declarch info`

**Solution:**
```bash
# Check file location
cat ~/.config/declarch/backends/mybackend.kdl

# Validate syntax
declarch check validate

# Check file permissions
ls -la ~/.config/declarch/backends/
```

### Binary Not Found

**Problem:** Backend fails with "command not found"

**Solution:**
```bash
# Test binary exists
which cargo

# Check if binary is in PATH
# Or specify full path:
binary "/usr/bin/cargo"
```

### List Parsing Fails

**Problem:** Packages not recognized as installed

**Solution:**
```bash
# Test list command manually
cargo install --list

# Adjust format and columns in backend config
```

## Best Practices

### 1. Use Standard Paths

Place backends in `~/.config/declarch/backends/`:

```bash
~/.config/declarch/backends/
├── cargo.kdl
├── npm.kdl
├── pip.kdl
└── custom.kdl
```

### 2. Document Requirements

Always include meta information:

```kdl
meta {
    title "My Backend"
    description "What this backend does"
    requires "required-binary"
}
```

### 3. Handle Errors Gracefully

Use fallback for common alternatives:

```kdl
binary "paru" "yay" "aura"
fallback "pacman"
```

### 4. Test Thoroughly

```bash
# Test each command
declarch sync --dry-run
declarch check
declarch info --backend mybackend
```

## Migration from v0.5

If you have backends defined in `declarch.kdl` using the old format:

**Old format (v0.5):**
```kdl
backends {
    custom {
        cmd "custom install"
        list_cmd "custom list"
        remove_cmd "custom remove"
    }
}
```

**New format (v0.6+):**
```kdl
// In ~/.config/declarch/backends/custom.kdl
backend "custom" {
    binary "custom"
    
    list "custom list" {
        format whitespace
        name_col 0
    }
    
    install "custom install {packages}"
    remove "custom remove {packages}"
}
```

Use `declarch init --backend <name>` to generate templates.

## Related

- [Init Command](../commands/init.md) - `declarch init --backend`
- [KDL Syntax Reference](../configuration/kdl-syntax.md) - Configuration syntax
- [Troubleshooting](troubleshooting.md) - Common issues
