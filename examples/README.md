# Declarch Configuration Examples

This directory contains example configurations for Declarch v0.6+.

## File Structure

```
examples/
├── declarch.kdl           # Main configuration example
├── backends.kdl           # Backend definitions (apt, aur, flatpak, npm)
├── modules/
│   └── base.kdl          # Base system module example
├── minimal.kdl           # Minimal single-file example
├── modular.kdl           # Modular configuration example
├── desktop.kdl           # Desktop environment example
└── development.kdl       # Development tools example
```

## Quick Start

1. Copy example files to your config directory:
   ```bash
   mkdir -p ~/.config/declarch/modules
   cp examples/backends/backends.kdl ~/.config/declarch/
   cp examples/declarch.kdl ~/.config/declarch/
   cp examples/modules/base.kdl ~/.config/declarch/modules/
   ```

2. Edit `~/.config/declarch/backends.kdl` to match your system (apt for Debian/Ubuntu, pacman for Arch, etc.)

3. Run `declarch sync` to apply the configuration

## Configuration Syntax

Declarch v0.6+ supports three package declaration syntaxes:

### 1. Nested Blocks (Recommended)
```kdl
pkg {
    apt {
        vim
        git
    }
    aur {
        paru
    }
}
```

### 2. Colon Syntax
```kdl
pkg:apt {
    vim
    git
}
```

### 3. Inline Prefix
```kdl
pkg {
    apt:vim
    apt:git
}
```

All three syntaxes can be mixed in the same file!

## Backend Configuration

See `backends/backends.kdl` for complete examples of:
- APT (Debian/Ubuntu)
- AUR Helper (paru/yay with pacman fallback)
- Flatpak
- NPM

Each backend defines:
- `binary`: Command to use
- `list`: How to list installed packages
- `install`/`remove`: Install/remove commands
- `format`: Output format (json, whitespace, tsv, regex)

## More Information

See the main project documentation for detailed syntax reference.
