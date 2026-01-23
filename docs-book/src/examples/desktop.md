# Desktop Environment Setup

Complete desktop configuration with multiple package managers.

## Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
    description "Hyprland desktop environment"
    host "desktop-arch"
}

// Import modules (optional)
imports {
    "modules/base"
    "modules/desktop/hyprland"
}

// === CONFLICTS ===
// Prevent installing competing packages
conflicts {
    i3    hyprland
    sway  hyprland
}

// === HOOKS ===
// Notifications during sync
on-pre-sync "notify-send 'Declarch' 'Starting package synchronization...'"
on-sync "notify-send 'Declarch' 'Packages updated successfully'"

// === SYSTEM PACKAGES (AUR) ===
packages {
    // Window manager
    hyprland
    waybar
    wofi

    // Terminal & Shell
    kitty
    zsh
    starship

    // Applications
    firefox
    thunar
    discord
}

// === CLI TOOLS (Static Binaries) ===
// Works on any Linux distro
packages:soar {
    bat     // Modern cat
    exa     // Modern ls
    ripgrep // Fast grep
    fd      // Fast find
    zoxide  // Smart cd
}

// === DEVELOPMENT TOOLS ===

// Rust crates
packages:cargo {
    ripgrep
    fd-find
    zoxide
    bat
    tealdeer
}

// Node.js packages
packages:npm {
    typescript
    prettier
    eslint
}

// === FLATPAK APPLICATIONS ===
packages:flatpak {
    com.spotify.Client
    org.telegram.desktop
    com.slack.Slack
    com.discordapp.Discord
    org.mozilla.firefox
}
```

## Features

### Desktop Environment
- **Hyprland** as Wayland compositor
- **Waybar** status bar
- **Wofi** application launcher
- **Kitty** terminal emulator
- **Zsh** + **Starship** prompt

### Applications
- **Firefox** browser
- **Thunar** file manager
- **Discord**, **Slack**, **Telegram** from Flatpak
- **Spotify** from Flatpak

### Development Tools
- **Rust tools**: ripgrep, fd, zoxide, bat via cargo
- **Node.js tools**: TypeScript, Prettier, ESLint via npm
- **Cross-distro CLI tools** via Soar

## What This Includes

### Window Manager & Desktop
- **Hyprland** - Dynamic tiling Wayland compositor
- **Waybar** - Customizable status bar
- **Wofi** - Application launcher for Wayland

### Cross-Distro Tools (Soar)
- **bat** - Cat with syntax highlighting
- **exa** - Better ls with colors
- **ripgrep** - Fast text search
- **fd** - Fast file search
- **zoxide** - Smart directory navigation

These work on **any Linux distro** (Ubuntu, Fedora, etc.) not just Arch!

### Development
- **Via cargo**: ripgrep, fd-find, zoxide, bat, tealdeer
- **Via npm**: typescript, prettier, eslint

### Flatpak Applications
- Spotify
- Telegram Desktop
- Slack
- Discord
- Firefox

## Usage

```bash
# Check configuration
declarch check

# Dry run to see what will change
declarch sync --dry-run

# Sync all packages
declarch sync
```

## Modular Setup

This config uses imports for organization:

```kdl
imports {
    "modules/base"
    "modules/desktop/hyprland"
}
```

Each file can be edited separately:
```bash
declarch edit base           # Edit modules/base.kdl
declarch edit hyprland       # Edit modules/desktop/hyprland.kdl
```

## Source Files

- [`desktop.kdl`](https://github.com/nixval/declarch/blob/main/examples/desktop.kdl)

---

**Next:** See [Development Environment](development.html) for programming-focused setup.
