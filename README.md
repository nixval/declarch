# declarch

> **Declarative Package Manager for Linux**

Define your packages once in config files, then sync your system with one command.

---

## Installation

### Arch Linux (AUR)

```bash
paru -S declarch
```

### Any Linux (Binary)

```bash
# One-liner install
curl -L https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-x86_64-unknown-linux-gnu.tar.gz | tar xz && sudo install declarch /usr/local/bin/
```

Or download manually:

```bash
wget https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-x86_64-unknown-linux-gnu.tar.gz
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/
```

### From Source

```bash
cargo install declarch --git https://github.com/nixval/declarch
```

---

## Quick Start

```bash
# Initialize config
declarch init

# Add packages
declarch install bat fzf ripgrep

# Sync system
declarch sync
```

---

## Configuration

Your config lives in `~/.config/declarch/`:

```
~/.config/declarch/
├── declarch.kdl       # Main config
├── state.json         # Installed packages
└── modules/           # Package modules
    ├── base.kdl
    ├── gaming.kdl
    └── work.kdl
```

### Example Config

```kdl
// modules/base.kdl
packages {
  bat          # Better cat
  fzf          # Fuzzy finder
  ripgrep      # Fast grep
  fd           # Better find
}

// Environment variables
env "EDITOR" {
  nvim
}

// Different package managers
packages:flatpak {
  com.spotify.Client
}

packages:npm {
  nodejs
}
```

---

## Common Commands

```bash
# Add packages to config
declarch install <package>

# Install specific backend
declarch install npm:nodejs

# Install to specific module
declarch install firefox --module browsers

# Sync system (install/remove packages)
declarch sync

# Sync with system update
declarch sync --update

# Show installed packages
declarch info

# Preview changes
declarch sync --dry-run
```

---

## Features

- **Declarative** - Define packages in KDL files
- **Multi-Backend** - AUR, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew
- **Modular** - Organize packages by purpose (base, gaming, work, etc.)
- **Cross-Distro** - Works on Arch, Debian, Fedora, and more
- **Git-Friendly** - Track configs in version control

---

## Cross-Distribution

| Backend | Arch | Debian | Fedora |
|---------|------|--------|--------|
| AUR     | ✓    | ✗      | ✗      |
| Flatpak | ✓    | ✓      | ✓      |
| npm     | ✓    | ✓      | ✓      |
| Custom  | ✓    | ✓      | ✓      |

For custom backends (nala, dnf, zypper, etc.), see [Cross-Distribution Support](https://nixval.github.io/declarch/cross-distro-support.html).

---

## Documentation

- **[Full Documentation](https://nixval.github.io/declarch/)** - Complete guide
- **[Getting Started](https://nixval.github.io/declarch/getting-started/quick-start.html)** - Detailed setup
- **[All Commands](https://nixval.github.io/declarch/commands/)** - Command reference

---

## License

MIT - see [LICENSE](LICENSE)
