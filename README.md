# declarch

> **Declarative Package Manager for Linux**

Define your packages once, sync everywhere. declarch manages your system packages through simple configuration files instead of running ad-hoc commands.

[![CI/CD](https://github.com/nixval/declarch/actions/workflows/ci.yml/badge.svg)](https://github.com/nixval/declarch/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## ✨ Quick Start

```bash
# Install (one-liner)
curl --proto '=https' --tlsv1.2 -sSf https://nixval.github.io/declarch/install.sh | sh

# Or from AUR (Arch Linux)
paru -S declarch

# Initialize with community config
declarch init shell/dms

# Install packages
declarch install bat fzf ripgrep

# Sync your system
declarch sync
```

---

## 📦 Installation

### Arch Linux (AUR)
```bash
paru -S declarch
# or
yay -S declarch
```

### Any Linux Distro (Binary)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://nixval.github.io/declarch/install.sh | sh
```

### From Source
```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

**📖 See [Installation Guide](https://nixval.github.io/declarch/getting-started/installation.html) for more options**

---

## 🚀 Basic Usage

### Install Packages
```bash
# Add to your config and install
declarch install bat fzf ripgrep

# Install specific backend
declarch install soar:bat npm:nodejs

# Install to specific module
declarch install brave --module browsers
```

### Sync System
```bash
# Sync all packages
declarch sync

# Sync with system update
declarch sync --update

# Sync specific module only
declarch sync --module base
```

### Manage Config
```bash
# Show installed packages
declarch info

# List all modules
declarch list modules

# Validate configuration
declarch check
```

---

## 🎯 Key Features

- **Declarative** - Define packages in KDL files, not ad-hoc commands
- **Multi-Backend** - AUR, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew, and custom backends
- **Modular** - Organize packages into logical modules (base, gaming, work, etc.)
- **Selective Sync** - Install specific modules without syncing everything
- **Auto Rollback** - Failed installations restore your config automatically
- **Cross-Distro** - Works on Arch, Debian, Fedora, and more

---

## 📚 Configuration

declarch stores config in `~/.config/declarch/`:

```
~/.config/declarch/
├── declarch.kdl          # Root configuration
├── state.json            # Installed packages state
└── modules/              # Package modules
    ├── base.kdl          # Base system packages
    ├── gaming.kdl        # Gaming packages
    └── work.kdl          # Work packages
```

**Example module:**
```kdl
// modules/base.kdl
packages {
  bat
  fzf
  ripgrep
  fd
}

// Environment variables
env "EDITOR" {
  nvim
}
```

**📖 See [Configuration Reference](https://nixval.github.io/declarch/configuration/kdl-syntax.html) for details**

---

## 🌐 Cross-Distribution Support

declarch works across multiple Linux distributions!

| Backend | Arch | Debian | Fedora | Other |
|---------|------|--------|--------|-------|
| AUR     | ✓    | ✗      | ✗      | ✗     |
| Flatpak | ✓    | ✓      | ✓      | ✓     |
| npm     | ✓    | ✓      | ✓      | ✓     |
| Custom  | ✓    | ✓      | ✓      | ✓     |

**📖 See [Cross-Distro Guide](https://nixval.github.io/declarch/cross-distro-support.html) for custom backends**

---

## 📖 Full Documentation

- **[Getting Started](https://nixval.github.io/declarch/getting-started/quick-start.html)** - New to declarch? Start here
- **[Commands Reference](https://nixval.github.io/declarch/commands/)** - All commands and options
- **[Configuration Guide](https://nixval.github.io/declarch/configuration/)** - KDL syntax and modules
- **[Hooks System](https://nixval.github.io/declarch/advanced/lifecycle-actions.html)** - Run scripts before/after sync
- **[Troubleshooting](https://nixval.github.io/declarch/advanced/troubleshooting.html)** - Common issues and solutions

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Documentation**: https://nixval.github.io/declarch/
- **GitHub**: https://github.com/nixval/declarch
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)
- **AUR (declarch)**: https://aur.archlinux.org/packages/declarch
- **AUR (declarch-bin)**: https://aur.archlinux.org/packages/declarch-bin
