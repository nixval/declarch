# Declarch Documentation

**Declarative package manager for Linux** — A comprehensive guide to managing your system packages with declarch.

## Overview

Declarch is a declarative package manager that provides a unified interface for managing packages across multiple package management backends. Instead of running imperative commands like `paru -S package` or `npm install package`, you declare your desired system state in a configuration file, and declarch automatically synchronizes your system to match that state.

### Key Concepts

- **Declarative Configuration**: Define what you want, not how to install it
- **Multi-Backend Support**: AUR, Flatpak, Soar, npm, pip, cargo, brew, yarn, pnpm, bun, and custom backends
- **Smart Synchronization**: Auto-installs missing packages, adopts existing ones
- **Safe Pruning**: Only removes packages that declarch manages
- **Module System**: Organize configurations into reusable modules
- **Remote Init**: Fetch configurations from GitHub/GitLab repositories

## Documentation

### Getting Started

| Guide | Description |
|-------|-------------|
| [Installation](getting-started/installation.md) | Install declarch on your system |
| [Quick Start](getting-started/quick-start.md) | Get up and running in 5 minutes |
| [Basic Concepts](getting-started/basic-concepts.md) | Understanding declarative package management |

### Commands

| Command | Description |
|---------|-------------|
| [`init`](commands/init.md) | Initialize or fetch configuration |
| [`sync`](commands/sync.md) | Synchronize packages with configuration |
| [`check`](commands/check.md) | Validate configuration and check for issues |
| [`info`](commands/info.md) | Show system status and managed packages |
| [`switch`](commands/switch.md) | Switch between package variants |
| [`edit`](commands/edit.md) | Edit configuration files |

### Configuration

| Topic | Description |
|-------|-------------|
| [KDL Syntax](configuration/kdl-syntax.md) | Complete KDL configuration reference |
| [Package Backends](configuration/backends.md) | Supported package managers and configuration |
| [Modules](configuration/modules.md) | Modular configuration organization |
| [Remote Init](advanced/remote-init.md) | Fetch configs from Git repositories |

### Advanced Topics

| Topic | Description |
|-------|-------------|
| [Hooks System](advanced/hooks.md) | Run commands before/after sync |
| [Custom Backends](advanced/custom-backends.md) | Add your own package managers |
| [Troubleshooting](advanced/troubleshooting.md) | Common issues and solutions |

### Examples

See the [examples/](../examples/) directory for real-world configuration examples:

- [minimal.kdl](../examples/minimal.kdl) - Basic setup
- [desktop.kdl](../examples/desktop.kdl) - Desktop environment
- [development.kdl](../examples/development.kdl) - Development tools
- [modular.kdl](../examples/modular.kdl) - Modular setup

## Quick Reference

### Basic Workflow

```bash
# Initialize
declarch init

# Edit configuration
declarch edit

# Preview changes
declarch sync --dry-run

# Apply changes
declarch sync
```

### Configuration Syntax

```kdl
// ~/.config/declarch/declarch.kdl

imports {
    modules/base
    modules/desktop
}

// Default backend (AUR)
packages {
    hyprland
    waybar
}

// Other backends
packages:flatpak {
    com.spotify.Client
}

packages:npm {
    typescript
    prettier
}
```

## Project Status

| Status | Description |
|--------|-------------|
| Version | v0.4.1 |
| License | MIT |
| Tested Distributions | Arch Linux, EndeavourOS |
| Cross-Distro Backends | Flatpak, Soar, npm, pip, cargo, brew (all Linux) |

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details.
