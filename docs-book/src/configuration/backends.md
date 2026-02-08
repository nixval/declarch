# Package Backends

Complete reference for package manager backends in declarch v0.6+.

## Overview

In v0.6+, declarch uses a **pure generic backend system**. All package managers are defined as KDL configuration files - no backend is hardcoded into the binary. This means:

- Any package manager can be supported by creating a backend file
- Backends are loaded from `~/.config/declarch/backends/*.kdl`
- Built-in backends are just default templates
- Users can override or extend any backend

## Architecture

### How Backends Work

1. **Backend Files**: Each backend is a KDL file in `~/.config/declarch/backends/`
2. **Generic Manager**: One unified `GenericManager` handles all backends
3. **Configuration-Driven**: Commands, parsers, and options are all configurable

### Backend Loading Order

1. Load built-in backend templates (embedded in binary)
2. Load user backends from `~/.config/declarch/backends/*.kdl`
3. User backends override built-in ones

## Quick Start

### Create a Backend

```bash
# Create a new backend (smart template)
declarch init --backend cargo

# Edit the generated file
nano ~/.config/declarch/backends/cargo.kdl
```

### Use a Backend

```kdl
// New v0.6+ syntax
pkg {
    cargo {
        ripgrep
        fd-find
        bat
    }
    npm {
        typescript
        prettier
    }
}
```

## Built-in Backend Templates

These backends come with pre-configured templates:

| Backend | Description | Requirements |
|---------|-------------|--------------|
| **aur** | Arch User Repository (paru/yay) | Arch Linux + paru/yay |
| **soar** | Static binary registry | Soar CLI |
| **flatpak** | Universal Linux apps | Flatpak |
| **npm** | Node.js packages | Node.js + npm |
| **yarn** | Yarn packages | Yarn |
| **pnpm** | pnpm packages | pnpm |
| **pip** | Python packages | Python + pip |
| **cargo** | Rust crates | Rust + Cargo |
| **gem** | Ruby gems | Ruby + gem |

## Backend File Structure

```kdl
// ~/.config/declarch/backends/<name>.kdl
backend "<name>" {
    meta {
        title "Display Name"
        description "What this backend does"
        version "1.0.0"
        author "your-name"
        tags "tag1" "tag2"
        homepage "https://..."
        requires "binary-name"
    }
    
    // Binary to use
    binary "binary-name" "alternative-binary"
    
    // Fallback backend if binary not found
    fallback "apt"
    
    // List installed packages
    list "binary list" {
        format tsv|whitespace|json|regex
        name_col 0
        version_col 1
    }
    
    // Install packages
    install "binary install {packages}"
    
    // Remove packages
    remove "binary remove {packages}"
    
    // Search packages (optional)
    search "binary search {query}" {
        format whitespace
        name_col 0
        desc_col 1
    }
    
    // Auto-confirmation flag (optional)
    noconfirm "-y"
    
    // Requires sudo (optional)
    needs_sudo true
}
```

## Configuration Syntax

### New v0.6+ Syntax

```kdl
pkg {
    backend1 {
        package1
        package2
    }
    backend2 {
        package3
        package4
    }
}
```

### Legacy v0.5 Syntax (Still Supported)

```kdl
// For default backend (AUR on Arch)
packages {
    package1
    package2
}

// For specific backend
packages:backend1 {
    package1
}
```

## Backend Reference

### AUR (Arch User Repository)

```bash
# Auto-detects paru or yay
declarch init --backend aur
```

```kdl
pkg {
    aur {
        hyprland
        waybar
        wofi
    }
}
```

### Soar (Static Binaries)

Cross-distribution static binary registry.

```bash
declarch init --backend soar
```

```kdl
pkg {
    soar {
        bat
        exa
        ripgrep
        fd
    }
}
```

### Flatpak

Universal application packages.

```bash
declarch init --backend flatpak
```

```kdl
pkg {
    flatpak {
        com.spotify.Client
        org.mozilla.firefox
        com.discordapp.Discord
    }
}
```

### NPM (Node.js)

```bash
declarch init --backend npm
```

```kdl
pkg {
    npm {
        typescript
        prettier
        eslint
    }
}
```

### Cargo (Rust)

```bash
declarch init --backend cargo
```

```kdl
pkg {
    cargo {
        ripgrep
        fd-find
        cargo-watch
    }
}
```

### pip (Python)

```bash
declarch init --backend pip
```

```kdl
pkg {
    pip {
        black
        ruff
        pytest
    }
}
```

## Cross-Distro Compatibility

These backends work on **any** Linux distribution:

| Backend | Works Everywhere | Notes |
|---------|-----------------|-------|
| **soar** | ✅ | Static binaries, no dependencies |
| **flatpak** | ✅ | Universal apps with sandboxing |
| **npm** | ✅ | Requires Node.js |
| **yarn** | ✅ | Requires Node.js |
| **pnpm** | ✅ | Requires Node.js |
| **pip** | ✅ | Requires Python |
| **cargo** | ✅ | Requires Rust |
| **gem** | ✅ | Requires Ruby |
| **brew** | ✅ | Requires Homebrew |

The **AUR** backend only works on Arch-based distributions.

## Default Backend

On first sync, declarch suggests a default backend based on your system:

- **Arch-based**: AUR (aur)
- **Debian/Ubuntu**: APT (via custom backend)
- **Fedora**: DNF (via custom backend)
- **Other**: Soar (soar)

## Custom Backends

### Creating a Backend

```bash
# Generate template
declarch init --backend nala

# Edit configuration
nano ~/.config/declarch/backends/nala.kdl
```

### Example: Nala (Debian/Ubuntu)

```kdl
backend "nala" {
    meta {
        title "Nala"
        description "APT frontend with better formatting"
        version "1.0.0"
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

### Example: DNF5 (Fedora)

```kdl
backend "dnf5" {
    meta {
        title "DNF5"
        description "Modern Fedora package manager"
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

See [Custom Backends Guide](../advanced/custom-backends.md) for complete documentation.

## Backend Availability

Declarch checks backend availability by looking for the configured binary:

```bash
$ declarch check

✓ AUR backend available (paru)
✓ Flatpak backend available
✓ npm backend available
✗ pip backend not available (pip3 not found)
✓ Cargo backend available
```

Packages for unavailable backends are skipped during sync.

## Mixing Backends

Use multiple backends in the same configuration:

```kdl
pkg {
    // System packages
    aur {
        hyprland
        waybar
    }
    
    // Cross-distro CLI tools
    soar {
        bat
        exa
        ripgrep
    }
    
    // Universal apps
    flatpak {
        com.spotify.Client
        org.mozilla.firefox
    }
    
    // Development tools
    npm {
        typescript
        prettier
    }
    
    pip {
        black
        ruff
    }
    
    cargo {
        fd-find
    }
}
```

## Package Name Conflicts

The same package name can exist in multiple backends:

```kdl
pkg {
    aur {
        ripgrep  // /usr/bin/ripgrep
    }
    cargo {
        ripgrep  // ~/.cargo/bin/ripgrep
    }
    soar {
        ripgrep  // ~/.local/bin/ripgrep
    }
}
```

Check for conflicts:
```bash
declarch check conflicts
```

Your PATH ordering determines which version runs!

## Performance

### Package Count Checks

| Backend | Check Speed (100 packages) |
|---------|---------------------------|
| AUR | ~2-3 seconds |
| Soar | ~1 second |
| Flatpak | ~3-4 seconds |
| npm | ~1-2 seconds |
| pip | ~1-2 seconds |
| Cargo | ~2-3 seconds |

### Optimization Tips

1. **Use targeted syncs:**
   ```bash
   declarch sync --target flatpak
   ```

2. **Group packages by backend in v0.6 syntax:**
   ```kdl
   // ✅ Efficient - grouped by backend
   pkg {
       npm { typescript prettier eslint }
       cargo { ripgrep fd-find }
   }
   ```

## Migration from v0.5

### Backend Definition

**Old (v0.5):** In `declarch.kdl`
```kdl
backends {
    custom {
        cmd "custom install"
        list_cmd "custom list"
        remove_cmd "custom remove"
    }
}
```

**New (v0.6):** Separate file in `backends/custom.kdl`
```kdl
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

### Package Syntax

**Old (v0.5):**
```kdl
packages:backend { pkg1 pkg2 }
```

**New (v0.6):**
```kdl
pkg {
    backend { pkg1 pkg2 }
}
```

Both syntaxes are supported for backward compatibility.

## Troubleshooting

### Backend Not Found

```bash
# Check backend exists
cat ~/.config/declarch/backends/<name>.kdl

# Validate syntax
declarch check validate
```

### Binary Not Found

```bash
# Test binary
which <binary>

# Or use full path in backend config
binary "/usr/bin/<binary>"
```

### List Parsing Fails

```bash
# Test list command
<binary> list

# Adjust format in backend config
list "<binary> list" {
    format whitespace|tsv|json|regex
    name_col 0
}
```

## Related

- [`declarch init --backend`](../commands/init.md) - Create backends
- [Custom Backends](../advanced/custom-backends.md) - Advanced backend configuration
- [KDL Syntax](kdl-syntax.md) - Configuration syntax
- [Modules](modules.md) - Organize configs
