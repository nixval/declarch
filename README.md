# ⚠️ BREAKING CHANGES IN v0.8.0

> **ESPECT SOME ERRORS** when upgrading from v0.5.x or earlier

## What Changed

- **New architecture**: Backend system completely rewritten
  - Official backends (aur, pacman, flatpak) are now built-in
  - Custom backends use import pattern
  - No more auto-loading from directories

- **New KDL syntax**:
  - Use `pkg` instead of `packages`
  - All string values must be quoted: `"whitespace"`, `"true"`
  - Backend blocks go inside `pkg { }`

- **New CLI structure**:
  - `declarch sync --update` → `declarch sync update`
  - `declarch sync --prune` → `declarch sync prune`
  - `declarch sync --dry-run` → `declarch sync preview`

## Migration Guide

```bash
# Backup your config
cp -r ~/.config/declarch ~/.config/declarch.backup

# Re-initialize (keeps your modules)
declarch init

# Fix syntax in your .kdl files:
# - Change 'packages {' to 'pkg {'
# - Change 'format whitespace' to 'format "whitespace"'
# - Change 'needs_sudo true' to 'needs_sudo "true"'

# Sync
declarch sync
```

---

# declarch

> **Declarative Package Manager for Linux**

Define your packages once in config files, then sync your system with one command.

```bash
# Initialize
$ declarch init
✓ Created config at ~/.config/declarch/

# Add packages
$ declarch install bat fzf ripgrep
✓ Added to modules/others.kdl

# Sync system
$ declarch sync
? Proceed with sync? [Y/n] y
✓ System synchronized
```

---

## Quick Start (3 minutes)

### 1. Install

```bash
# Arch Linux (AUR)
paru -S declarch

# Any Linux (Binary)
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### 2. Initialize

```bash
declarch init
```

This creates:
```
~/.config/declarch/
├── declarch.kdl       # Main config
├── backends.kdl       # Backend definitions (aur, pacman, flatpak)
└── modules/
    └── base.kdl       # Your packages
```

### 3. Add Packages

```bash
# Add to default module
declarch install bat fzf ripgrep

# Add to specific backend
declarch install aur:neovim

# Add to specific module
declarch install firefox --module browsers
```

### 4. Sync

```bash
declarch sync
```

---

## Configuration

Edit `~/.config/declarch/modules/base.kdl`:

```kdl
pkg {
    // Official backends (aur, pacman, flatpak) work out of the box
    aur {
        neovim
        bat
        exa
    }
    
    pacman {
        firefox
        thunderbird
    }
    
    flatpak {
        com.spotify.Client
        com.discordapp.Discord
    }
}
```

---

## Common Commands

```bash
# Add packages
declarch install <package>

# Add with backend prefix
declarch install npm:typescript

# Sync system
declarch sync

# Preview changes (dry-run)
declarch sync preview

# Sync + update system packages
declarch sync update

# Remove unmanaged packages
declarch sync prune

# Search packages
declarch search firefox

# Check config
declarch check
```

---

## Custom Backends

Need npm, cargo, or other package managers?

```bash
# Add npm backend
declarch init --backend npm
✓ Backend 'npm' adopted!

# Use it
declarch install npm:prettier
```

---

## Features

- **Declarative** - Define packages in KDL files
- **Multi-Backend** - AUR, pacman, flatpak, npm, cargo, pip, and more
- **Modular** - Split into multiple files (base, gaming, work, etc.)
- **Smart Errors** - Clear error messages with line numbers
- **Fast** - Only installs what you need

---

## Documentation

- **[Full Docs](https://nixval.github.io/declarch/)** - Complete guide
- **[Getting Started](https://nixval.github.io/declarch/getting-started/quick-start.html)** - Detailed walkthrough
- **[Command Reference](https://nixval.github.io/declarch/commands/)** - All commands

---

## License

MIT - see [LICENSE](LICENSE)
