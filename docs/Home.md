# Declarch

**Declarative Package Manager for Linux**

A declarative package manager for Linux, inspired by Nix workflow. Declarch allows you to define your desired system state in a simple KDL configuration file and automatically install/remove packages to match that state.

---

## Features

- 🎯 **Declarative Configuration** - Define what you want, not how to install it
- 📦 **Multi-Backend Support** - AUR (Arch), Flatpak (universal), Soar (cross-distro)
- 🔧 **Cross-Distro** - Works on any Linux distribution
- 📝 **Simple Syntax** - Human-readable KDL format
- 🌍 **Remote Init** - Fetch configs from GitHub/GitLab repositories
- 🧩 **Modular** - Organize configs into reusable modules
- 🔄 **Idempotent** - Safe to run multiple times

---

## Quick Example

Create `declarch.kdl`:

```kdl
packages:aur {
    hyprland
    waybar
}

packages {
    bat
    exa
    ripgrep
}

packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

Apply:

```bash
declarch sync
```

---

## Installation

### From AUR

```bash
paru -S declarch
# or
yay -S declarch
```

### From Source

```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

### Quick Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

---

## Quick Start

### 1. Initialize Config

```bash
# Create empty config
declarch init

# Or fetch from remote
declarch init myuser/dotfiles
```

### 2. Edit Configuration

```bash
declarch edit
```

### 3. Preview Changes

```bash
declarch check
```

### 4. Apply

```bash
declarch sync
```

---

## Advanced Features

### Hooks (Automation)

Run commands before/after sync:

```kdl
on-pre-sync "echo 'Starting sync...'"
on-sync "notify-send 'Packages updated'"
on-sync-sudo "systemctl restart gdm"
```

**Security**: Hooks are disabled by default. Enable with `--hooks` flag:

```bash
declarch sync --hooks      # Execute hooks
declarch sync --dry-run    # Preview changes (shows hooks)
```

**Why disabled by default?**
- Remote configs may contain arbitrary commands
- Review the config before enabling hooks
- Dry-run always shows what would be executed

---

## Documentation

- [Installation Guide](Installation.md) - Detailed installation instructions
- [Quick Start Guide](Quick-Start.md) - First steps with declarch
- [KDL Syntax Reference](KDL-Syntax-Reference.md) - Complete syntax documentation
- [Remote Init Guide](Remote-Init-Guide.md) - Fetch configs from GitHub/GitLab
- [Repository Requirements](Repository-Requirements.md) - Make your repo declarch-ready
- [Examples](Examples.md) - Real-world configuration examples

---

## Supported Package Backends

| Backend | Description | Distro | Syntax |
|---------|-------------|--------|--------|
| **Soar** | Static binaries (cross-distro) | All Linux | `packages { bat }` |
| **AUR** | Arch User Repository | Arch-based | `packages:aur { hyprland }` |
| **Flatpak** | Universal packages | All Linux | `packages:flatpak { com.spotify.Client }` |

---

## Comparison with Alternatives

| Feature | Declarch | Nix | Guix | Ansible |
|---------|----------|-----|------|---------|
| Declarative | ✅ | ✅ | ✅ | ✅ |
| Native Packages | ✅ | ❌ | ❌ | ❌ |
| AUR Support | ✅ | ❌ | ❌ | ❌ |
| Learning Curve | Low | High | High | Medium |
| Cross-Distro | Partial | ✅ | ✅ | ❌ |
| Binary Size | Small | Large | Large | N/A |

---

## Use Cases

### Personal Dotfiles

Keep your package list in git and sync across machines:

```bash
git clone https://github.com/username/dotfiles.git
cd dotfiles
declarch sync
```

### Team/Company Standardization

Share standard development environment:

```bash
declarch init company/dev-environment
declarch sync
```

### Multiple Machine Configs

Different configs for desktop, laptop, server:

```kdl
// Base packages for all machines
packages {
    bat
    exa
}

// Host-specific
imports {
    hosts/desktop
    hosts/laptop
}
```

---

## Community

- **GitHub**: https://github.com/nixval/declarch
- **Issues**: https://github.com/nixval/declarch/issues
- **Config Registry**: https://github.com/nixval/declarch-packages

---

## License

MIT License - see [LICENSE](https://github.com/nixval/declarch/blob/main/LICENSE)

---

## Contributing

Contributions are welcome! Please read [Contributing Guidelines](https://github.com/nixval/declarch/blob/main/CONTRIBUTING.md)

---

**Made with ❤️ for Arch Linux users**
