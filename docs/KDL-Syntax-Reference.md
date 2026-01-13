# Declarch KDL Syntax Reference

Complete reference for all valid KDL syntax in declarch configuration files.

---

## Table of Contents

1. [Package Declarations](#package-declarations)
2. [Imports](#imports)
3. [Configuration Settings](#configuration-settings)
4. [Package Management](#package-management)
5. [Comments](#comments)
6. [Complete Examples](#complete-examples)

---

## Package Declarations

### 1. Soar Packages (Cross-Distro Static Binaries)

**Syntax:**
```kdl
packages {
    bat
    exa
    ripgrep
    fd
}
```

**Or with colon syntax:**
```kdl
packages:soar {
    bat
    exa
}
```

**Description:** Static binaries from Soar registry. Works on any Linux distribution.

---

### 2. AUR Packages (Arch Linux Only)

**Syntax:**
```kdl
packages:aur {
    hyprland
    waybar
    swww
}
```

**Or legacy syntax:**
```kdl
aur-packages {
    hyprland
    waybar
}
```

**Description:** Packages from Arch User Repository (AUR). Only available on Arch-based distributions.

---

### 3. Flatpak Packages (Cross-Distro)

**Syntax:**
```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    org.telegram.desktop
}
```

**Or legacy syntax:**
```kdl
flatpak-packages {
    com.spotify.Client
}
```

**Description:** Flatpak applications. Works on any Linux distribution with Flatpak installed.

---

### 4. Embedded Syntax (Mixed Backends)

**Syntax:**
```kdl
packages {
    // Soar packages (default)
    bat
    exa

    // Embedded backend-specific packages
    aur {
        hyprland
        waybar
    }

    flatpak {
        com.spotify.Client
        org.mozilla.firefox
    }
}
```

**Description:** Mix different package types in one block. Default (unnamed) items are Soar packages.

---

## Imports

### Import Modules

**Syntax:**
```kdl
imports {
    modules/hyprland/niri-nico
    modules/gaming/steam-setup
}
```

**Or single import:**
```kdl
imports modules/hyprland/niri-nico
```

**Or legacy syntax:**
```kdl
import modules/hyprland/niri-nico
```

**Description:** Import other KDL files. Paths are relative to the current file.

**Examples:**
```kdl
// Import from modules/ directory
imports modules/hyprland/niri-nico

// Import from home directory
imports ~/dotfiles/declarch/my-config.kdl

// Import from absolute path
imports /usr/share/declarch/common.kdl
```

---

## Configuration Settings

### Editor

**Syntax:**
```kdl
editor "nvim"
```

**Or:**
```kdl
editor nvim
```

**Description:** Set the default editor for `declarch edit` command.

**Priority:**
1. `editor "nvim"` in declarch.kdl
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default fallback)

**Examples:**
```kdl
editor "nvim"      // Neovim
editor "vim"       // Vim
editor "code"      // VS Code
editor "micro"     // Micro
```

---

### Description (No-Op)

**Syntax:**
```kdl
description {
    This is my personal Arch Linux setup
    with Hyprland window manager and Waybar status bar
}
```

**Or single-line:**
```kdl
description "My personal Hyprland setup"
```

**Description:** Add documentation/comments to your config. This is a no-op node that doesn't affect package installation. It's purely for documentation purposes.

---

## Package Management

### Exclude Packages

**Syntax:**
```kdl
excludes {
    unwanted-package
    another-package
}
```

**Or:**
```kdl
excludes unwanted-package another-package
```

**Description:** Exclude specific packages from being installed, even if they're listed in imported modules.

---

### Package Aliases

**Syntax:**
```kdl
aliases-pkg {
    pipewire pipewire-jack2
    python-poetry python-poetry-core
}
```

**Or inline:**
```kdl
aliases-pkg pipewire pipewire-jack2
```

**Description:** Map a config package name to an actual package name. Useful for variants or alternative packages.

**Example:**
```kdl
aliases-pkg {
    // In config: "pipewire" → Installs: "pipewire-jack2"
    pipewire pipewire-jack2

    // In config: "firefox-beta" → Installs: "firefox"
    firefox-beta firefox
}
```

---

## Comments

### Single-Line Comments

**Syntax:**
```kdl
// This is a comment
packages {
    bat  // This is also a comment
}
```

**Description:** Single-line comments start with `//` and continue to the end of the line.

---

### Multi-Line Comments

**KDL doesn't support multi-line comments.** Use multiple single-line comments:

```kdl
// Comment line 1
// Comment line 2
// Comment line 3
packages {
    bat
}
```

---

## Complete Examples

### Minimal Config

```kdl
// My minimal setup
editor "nvim"

packages {
    bat
    exa
    fd
    ripgrep
}
```

---

### Full Desktop Setup

```kdl
// My Hyprland Desktop Setup
// Author: Your Name
// Source: https://github.com/username/dotfiles

// Configuration
editor "nvim"

description {
    Full Hyprland desktop setup with Waybar, SDDM, and gaming support
}

// Core AUR packages
packages:aur {
    hyprland
    waybar
    swww
    wlogout
    wofi
    hyprpaper
    hyprpicker
}

// Flatpak applications
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    org.telegram.desktop
    com.discordapp.Discord
}

// CLI tools (Soar)
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    zoxide
    delta
}

// Gaming setup
imports {
    modules/gaming/steam-setup
}

// Package aliases
aliases-pkg {
    pipewire pipewire-jack2
}

// Exclude unwanted packages
excludes {
    nano
    vi
}
```

---

### Development Environment

```kdl
// Development Environment Config
editor "nvim"

description "Cross-platform development setup"

// Node.js development
packages:npm {
    typescript
    prettier
    eslint
}

// Rust development
packages:cargo {
    ripgrep
    fd-find
    eza
    git-delta
}

// Python development
packages:python {
    ruff
    black
    poetry
}
```

---

### Multi-Host Setup

**Root config (`~/.config/declarch/declarch.kdl`):**
```kdl
// Base packages for all hosts
packages {
    bat
    exa
    fd
    ripgrep
}

// Host-specific modules
imports {
    hosts/desktop
    hosts/laptop
}
```

**Host-specific module (`hosts/desktop.kdl`):**
```kdl
packages:aur {
    hyprland
    waybar
}

packages:flatpak {
    com.spotify.Client
}
```

---

## Best Practices

### 1. Use Descriptive Names

```kdl
// Good
packages {
    bat
    exa
    ripgrep
}

// Avoid - unclear what these packages do
packages {
    pkg1
    pkg2
    pkg3
}
```

---

### 2. Group Related Packages

```kdl
// Good - organized by purpose
packages {
    // CLI tools
    bat
    exa
    fd
    ripgrep
}

packages:aur {
    // Window manager
    hyprland
    waybar
    swww
}

packages:flatpak {
    // Communication
    org.telegram.desktop
    com.discordapp.Discord
}
```

---

### 3. Use Aliases for Variants

```kdl
// Instead of:
packages:aur {
    pipewire-jack2
}

// Use alias for clarity:
aliases-pkg {
    pipewire pipewire-jack2
}

packages:aur {
    pipewire  // Will install pipewire-jack2
}
```

---

### 4. Document Your Config

```kdl
// My Personal Arch Setup
// Last updated: 2025-01-13
// Source: https://github.com/username/dotfiles

description {
    Personal Arch Linux setup focused on:
    - Minimalism (only what I need)
    - Hyprland WM (tiling, Wayland)
    - Gaming (Steam, Lutris)
    - Development (Rust, Node.js, Python)
}

editor "nvim"  // Set default editor
```

---

## Troubleshooting

### Invalid Syntax

**Error:** `KDL syntax error`

**Common issues:**
1. Missing closing brace `}`
2. Unmatched quotes `"`
3. Invalid node names (use lowercase, hyphens only)
4. Trailing commas (KDL doesn't use commas)

**Example of invalid syntax:**
```kdl
packages {  // Missing closing brace
    bat,
    exa,    // KDL doesn't use commas
}
```

**Correct syntax:**
```kdl
packages {
    bat
    exa
}
```

---

### Package Not Found

**Error:** `Package not found`

**Solutions:**
1. Check package name spelling
2. Verify package exists in the specified backend
3. For AUR: Make sure AUR helper is installed
4. For Flatpak: Make sure Flathub is added

---

### Import Path Not Found

**Error:** `Module not found`

**Solutions:**
1. Verify file exists at the specified path
2. Check file extension (should be `.kdl`)
3. Use relative paths from current file
4. For absolute paths: start with `/` or `~/`

---

## Summary

### Valid Top-Level Nodes

| Node | Syntax | Purpose |
|------|--------|---------|
| `packages` | `packages { ... }` | Soar packages (default) |
| `packages:aur` | `packages:aur { ... }` | AUR packages |
| `packages:flatpak` | `packages:flatpak { ... }` | Flatpak packages |
| `imports` | `imports { ... }` | Import modules |
| `excludes` | `excludes { ... }` | Exclude packages |
| `aliases-pkg` | `aliases-pkg { ... }` | Package aliases |
| `editor` | `editor "nvim"` | Set editor |
| `description` | `description { ... }` | Documentation (no-op) |

### Legacy Syntax (Still Supported)

| Old Syntax | New Syntax | Status |
|-----------|-----------|--------|
| `aur-packages { ... }` | `packages:aur { ... }` | Deprecated but works |
| `flatpak-packages { ... }` | `packages:flatpak { ... }` | Deprecated but works |
| `import ...` | `imports ...` | Deprecated but works |

---

## See Also

- [Declarch README](https://github.com/nixval/declarch)
- [KDL Language Spec](https://kdl.dev)
- [Remote Init Guide](./REMOTE_INIT_GUIDE.md)
- [Repository Requirements](./REPO_REQUIREMENTS.md)
