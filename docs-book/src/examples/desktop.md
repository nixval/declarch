# Desktop Environment Setup

A complete desktop configuration for daily use.

## Quick Example

```kdl
meta {
  host "desktop-arch"
  description "My desktop computer"
}

// Window manager & desktop
packages {
  hyprland    // Tiling Wayland compositor
  waybar      // Status bar
  wofi        // App launcher
  kitty       // Terminal
  zsh         // Shell
}

// Cross-distro tools (work on ANY Linux!)
packages:soar {
  bat     // Better cat
  exa     // Better ls
  ripgrep // Fast search
  fd      // Fast find
}

// Development tools
packages:cargo {
  ripgrep
  fd-find
}

packages:npm {
  typescript
  prettier
}

// Desktop apps (from Flatpak)
packages:flatpak {
  com.spotify.Client        // Music
  org.mozilla.firefox       // Browser
  com.discordapp.Discord    // Chat
}
```

## What This Includes

### Desktop Environment
- **Hyprland** - Modern Wayland tiling window manager
- **Waybar** - Customizable status bar
- **Wofi** - Application launcher
- **Kitty** - Fast GPU-accelerated terminal

### Cross-Distro Tools
These work on **Arch, Ubuntu, Fedora, ANY Linux**:
- **bat** - `cat` with colors and syntax highlighting
- **exa** - `ls` with colors and git info
- **ripgrep** - Ultra-fast text search
- **fd** - Fast, user-friendly alternative to `find`

### Development Tools
- Via **cargo**: ripgrep, fd-find
- Via **npm**: typescript, prettier

### Apps
- **Spotify** - Music streaming
- **Firefox** - Web browser
- **Discord** - Chat/Voice

## Usage

```bash
# See what will be installed
declarch check

# Dry-run (preview changes)
declarch sync --dry-run

# Install everything
declarch sync
```

## Supported Package Managers

Declarch works with many package managers. Here are the most common:

| Package Manager | Backend | Use For |
|----------------|---------|---------|
| Arch/AUR | `packages` | System packages |
| Soar | `packages:soar` | Cross-distro tools |
| npm | `packages:npm` | Node.js tools |
| cargo | `packages:cargo` | Rust crates |
| Flatpak | `packages:flatpak` | Desktop apps |

## Three Syntax Styles

You can write packages in 3 different ways. All work the same:

**Style 1: Backend blocks** (recommended, clearest):
```kdl
packages:npm {
  typescript
  prettier
}
```

**Style 2: Embedded blocks**:
```kdl
packages {
  npm {
    typescript
    prettier
  }
}
```

**Style 3: Inline prefix**:
```kdl
packages {
  npm:typescript
  npm:prettier
}
```

Mix them however you like!

## Source Files

- [`desktop.kdl`](https://github.com/nixval/declarch/blob/main/examples/desktop.kdl)

---

**Next:** See [Development Environment](development.html) for programming setup.
