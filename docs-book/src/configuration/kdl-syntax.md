# KDL Syntax Reference

Complete reference for KDL (Key Definition Language) syntax in declarch configuration files.

## Table of Contents

- [Basic Structure](#basic-structure)
- [Package Declarations](#package-declarations)
- [Backend Syntax](#backend-syntax)
- [Comments](#comments)
- [Advanced Features](#advanced-features)
- [Complete Example](#complete-example)

## Basic Structure

### Configuration File

Location: `~/.config/declarch/declarch.kdl`

```kdl
// This is a comment
// ~/.config/declarch/declarch.kdl

// Imports
imports {
    modules/base
    modules/desktop
}

// Package declarations
packages {
    bat
    exa
    ripgrep
}
```

### Module Files

Location: `~/.config/declarch/modules/<name>.kdl`

```kdl
// ~/.config/declarch/modules/desktop.kdl

packages {
    hyprland
    waybar
}

packages:flatpak {
    com.spotify.Client
}
```

## Package Declarations

### Simple List

```kdl
packages {
    bat
    exa
    ripgrep
    fd
    fzf
}
```

### Horizontal List

```kdl
packages {
    bat exa ripgrep fd fzf
}
```

### Mixed (Both Styles)

```kdl
packages {
    // Vertical for clarity
    hyprland
    waybar

    // Horizontal for related groups
    rofi kitty wl-clipboard

    // Back to vertical
    wlogout
}
```

### Single Package

```kdl
packages {
    bat
}
```

## Backend Syntax

### Default Backend (AUR on Arch)

```kdl
packages {
    bat          // Uses default backend (AUR on Arch)
    exa
    ripgrep
}
```

### Explicit Backend

```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}

packages:npm {
    typescript
    prettier
}

packages:pip {
    black
    ruff
}

packages:cargo {
    ripgrep
    fd-find
}
```

### Inline Backend Syntax

```kdl
packages {
    aur:hyprland          // Explicit AUR
    flatpak:com.spotify.Client  // Explicit Flatpak
    npm:typescript        // Explicit npm
    pip:black             // Explicit pip
}
```

### Mixed Backend Declaration

```kdl
packages {
    // Default backend (AUR)
    hyprland
    waybar

    // Inline backend syntax
    flatpak:com.spotify.Client
    npm:typescript

    // Embedded block
    npm {
        prettier
        eslint
    }

    // Another embedded block
    python {
        black
        ruff
    }
}
```

## Comments

### Single Line

```kdl
// This is a comment
packages {
    bat  // End-of-line comment
}
```

### Block Comments

KDL doesn't have multi-line block comments. Use single-line comments:

```kdl
// This is line 1 of comment
// This is line 2 of comment
packages {
    bat
}
```

### Documentation Comments

```kdl
// === DESKTOP ENVIRONMENT ===
// This section contains all desktop-related packages

packages {
    hyprland    // Window manager
    waybar      // Status bar
}
```

## Advanced Features

### Imports

```kdl
imports {
    modules/base
    modules/desktop
    modules/development

    // Nested modules
    modules/development/rust
    modules/development/python
}
```

### Project Metadata

```kdl
meta {
    description "My Hyprland Workstation"
    author "nixval"
    version "1.0.0"
    tags "workstation" "hyprland" "development"
    url "https://github.com/nixval/dotfiles"
}
```

### Editor

```kdl
editor "nvim"
```

Used by `declarch edit` command.

### Conflicts

Define mutually exclusive packages:

```kdl
conflicts {
    vim neovim
    pipewire pulseaudio
    firefox firefox-beta
}
```

### Backend Options

Configure package manager behavior:

```kdl
options:aur {
    noconfirm
    helper "paru"
}

options:flatpak {
    noconfirm
}
```

### Environment Variables

```kdl
// Global environment
env EDITOR="nvim"
env VISUAL="nvim"

// Backend-specific
env:aur MAKEFLAGS="-j4"
env:aur CARGO_HOME="$HOME/.cargo"
```

### Package Sources

Add custom package sources:

```kdl
repos:aur {
    "https://aur.archlinux.org"
}

repos:flatpak {
    "https://flathub.org/repo/flathub.flatpakrepo"
}
```

### Policy

Define package lifecycle policies:

```kdl
policy {
    protected {
        linux
        systemd
        base-devel
    }
    orphans "keep"  // or "remove" or "ask"
}
```

### Lifecycle Actions

Run commands before/after sync:

```kdl
// Pre-sync action
hooks {
    pre-sync "notify-send 'Starting sync...'"
}

// Post-sync action
hooks {
    post-sync "notify-send 'Packages updated'"
}

// Post-sync with sudo
hooks {
    post-sync "systemctl restart gdm" --sudo
}
```

### Package Mappings

```kdl
aliases-pkg {
    pipewire pipewire-jack2
}
```

### Excludes

Exclude packages from management:

```kdl
excludes {
    nano
    vi
}
```

## Supported Backends

### AUR (Arch User Repository)

Default on Arch-based systems.

```kdl
packages {
    hyprland
    waybar
}

// Or explicitly
packages:aur {
    hyprland
}
```

### Soar (Static Binaries)

Cross-distro static binary registry.

```kdl
packages:soar {
    bat
    exa
    ripgrep
}
```

### Flatpak (Universal Apps)

```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    com.discordapp.Discord
}
```

### npm (Node.js)

```kdl
packages:npm {
    typescript
    prettier
    eslint
}
```

### Python

```kdl
packages:pip {
    black
    ruff
    jupyter
}
```

### Cargo (Rust)

```kdl
packages:cargo {
    ripgrep
    fd-find
    cargo-edit
}
```

### Yarn

```kdl
packages:yarn {
    eslint
    prettier
}
```

### pnpm

```kdl
packages:pnpm {
    typescript
    prettier
}
```

### Bun

```kdl
packages:bun {
    @types/node
    prettier
}
```

### Brew (Homebrew)

```kdl
packages:brew {
    vim
    ffmpeg
}
```

## Syntax Rules

### Identifiers

Package names must be valid identifiers:

```
- Letters: a-z, A-Z
- Numbers: 0-9
- Special: -, _, @, .

Examples:
✓ bat
✓ ripgrep
✓ @types/node
✓ com.spotify.Client
✓ python-black

✗ pkg with spaces
✗ pkg@special#chars
```

### Strings

Use strings for special characters:

```kdl
meta {
    description "My 'Workstation' Setup"
}
```

### Whitespace

KDL is whitespace-insensitive. All these are equivalent:

```kdl
// Style 1: Vertical
packages {
    bat
    exa
}

// Style 2: Horizontal
packages { bat exa }

// Style 3: Mixed
packages {
    bat exa
    ripgrep fd
}
```

### Nesting

Blocks can be nested:

```kdl
packages {
    // Nested block
    npm {
        typescript
        prettier
    }
}
```

## Complete Example

```kdl
// ~/.config/declarch/declarch.kdl

// === METADATA ===
meta {
    description "My Hyprland Workstation"
    author "nixval"
    version "1.0.0"
    tags "workstation" "hyprland" "development"
}

// === EDITOR ===
editor "nvim"

// === IMPORTS ===
imports {
    modules/base
    modules/desktop
    modules/development
}

// === CONFLICTS ===
conflicts {
    vim neovim
    pipewire pulseaudio
}

// === POLICY ===
policy {
    protected {
        linux
        systemd
        base-devel
    }
    orphans "keep"
}

// === BACKEND OPTIONS ===
options:aur {
    noconfirm
    helper "paru"
}

// === ENVIRONMENT ===
env EDITOR="nvim"
env:aur MAKEFLAGS="-j4"

// === LIFECYCLE ACTIONS ===
hooks {
    pre-sync "notify-send 'Starting sync...'"
    post-sync "notify-send 'Packages updated'"
}

// === PACKAGES ===

// Default backend (AUR)
packages {
    hyprland
    waybar
    wofi
    kitty
    rofi
    wl-clipboard
}

// Static binaries (cross-distro)
packages:soar {
    bat
    exa
    ripgrep
    fd
    fzf
}

// Universal apps
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    com.discordapp.Discord
    org.telegram.desktop
}

// Node.js ecosystem
packages:npm {
    typescript
    prettier
    eslint
}

// Python ecosystem
packages:pip {
    black
    ruff
    jupyter
}

// Rust ecosystem
packages:cargo {
    ripgrep
    fd-find
    cargo-edit
    cargo-watch
}
```

## Style Guide

### Recommended Style

```kdl
// ✅ Good: Clear, organized
packages {
    // System utilities
    bat
    exa
    ripgrep

    // Desktop apps
    hyprland
    waybar
}

// ✅ Good: Grouped by purpose
packages:npm {
    // Formatters
    prettier
    eslint

    // Languages
    typescript
}
```

### Avoid

```kdl
// ❌ Bad: No organization
packages { bat exa ripgrep hyprland waybar wofi kitty rofi npm typescript prettier }

// ❌ Bad: Inconsistent
packages {
    bat exa
    ripgrep
    fd fzf
}
```

## Best Practices

1. **Group related packages:**
```kdl
packages {
    // CLI tools
    bat exa ripgrep

    // Desktop
    hyprland waybar
}
```

2. **Use comments for clarity:**
```kdl
packages {
    hyprland    // Window manager
    waybar      // Status bar
}
```

3. **Keep lists manageable:**
```kdl
// ✅ Use modules for large lists
imports {
    modules/development
    modules/desktop
}
```

4. **Be explicit with backends when needed:**
```kdl
packages {
    aur:hyprland
    flatpak:com.spotify.Client
}
```

## Related

- [Backends Reference](backends.md) - All supported backends
- [Modules Guide](modules.md) - Modular configuration
- [Advanced Topics](../advanced/) - Hooks, custom backends, etc.
