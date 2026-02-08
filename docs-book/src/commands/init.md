# declarch init

Initialize a new declarch configuration, module, or backend.

## Usage

```bash
# Initialize root configuration
declarch init

# Initialize from a remote source
declarch init <SOURCE>

# Create a new module
declarch init <module-name>

# Create a new backend (v0.6+)
declarch init --backend <name>
```

## Quick Start

### Empty Configuration

```bash
declarch init
```

Creates a fresh, empty configuration at `~/.config/declarch/`.

### Fetch from GitHub

```bash
declarch init shell/dms
```

Downloads and uses a pre-made configuration from GitHub.

### Using Direct URL

```bash
declarch init https://example.com/config.kdl
```

Loads configuration from any URL.

## Options

| Option | Description |
|--------|-------------|
| `--host <NAME>` | Use hostname-specific config |
| `--backend <NAME>` | Create a new backend configuration (v0.6+) |
| `--force` | Overwrite existing files |
| `--skip-soar-install` | Skip automatic Soar installation |

## Creating Backends (v0.6+)

Create custom package manager backends:

```bash
# Create a backend for cargo (Rust)
declarch init --backend cargo

# Create a backend for npm (Node.js)
declarch init --backend npm

# Create a custom backend
declarch init --backend my-custom-pm
```

This creates a backend configuration file at:
```
~/.config/declarch/backends/<name>.kdl
```

Edit the file to customize:
- Binary name
- Install/remove/list commands
- Output parsing format
- Search configuration

See [Custom Backends](../advanced/custom-backends.md) for detailed backend configuration.

## Creating Modules

```bash
# Create a simple module
declarch init base

# Create a nested module
declarch init desktop/hyprland

# With auto-import
declarch init development --force
```

Modules are created in `~/.config/declarch/modules/` and automatically imported to your main config.

## What It Does

### Root Initialization

1. Creates `~/.config/declarch/` directory
2. Sets up `declarch.kdl` configuration file
3. Initializes package state tracking
4. Creates `modules/` directory for organization
5. Creates `backends/` directory for custom backends

### Module Initialization

1. Creates module file in `modules/`
2. Auto-injects import to root config
3. Uses template if available (or creates default)

### Backend Initialization (v0.6+)

1. Creates backend file in `backends/`
2. Generates template based on backend name
3. Includes smart defaults for known package managers

## Common Source Formats

| Format | Example | Description |
|--------|---------|-------------|
| GitHub repo | `myuser/dotfiles` | Fetch from GitHub |
| With variant | `myuser/dotfiles:hyprland` | Fetch specific variant |
| With branch | `myuser/dotfiles/main` | Fetch from branch |
| GitLab | `gitlab.com/user/repo` | Fetch from GitLab |
| Direct URL | `https://example.com/config.kdl` | Load from URL |

## Examples

### Initialize with Hostname

```bash
declarch init --host laptop
```

Creates configuration with laptop-specific settings.

### Force Overwrite

```bash
declarch init --force
declarch init --backend cargo --force
```

Overwrites existing configuration or backend.

### Skip Soar Installation

```bash
declarch init --skip-soar-install
```

Skip automatic Soar package manager installation.

## Next Steps

After init:

```bash
# Review the config
cat ~/.config/declarch/declarch.kdl

# Add packages
declarch install bat fzf ripgrep

# Create a backend for cargo
declarch init --backend cargo

# Sync your system
declarch sync
```

## Related

- [`install` (command)](../commands/install.md) - Add packages to config
- [`sync`](sync.md) - Apply configuration to system
- [Custom Backends](../advanced/custom-backends.md) - Backend configuration guide
- [Remote Init Guide](../advanced/remote-init.md) - Advanced remote configurations
