# declarch init

Initialize a new declarch configuration.

## Usage

```bash
declarch init [SOURCE]
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

## What It Does

1. Creates `~/.config/declarch/` directory
2. Sets up `declarch.kdl` configuration file
3. Initializes package state tracking
4. Creates `modules/` directory for organization

## Common Source Formats

| Format | Example |
|--------|---------|
| GitHub repo | `myuser/dotfiles` |
| With variant | `myuser/dotfiles:hyprland` |
| Direct URL | `https://example.com/config.kdl` |

## Next Steps

After init:

```bash
# Review the config
cat ~/.config/declarch/declarch.kdl

# Add packages
declarch install bat fzf ripgrep

# Sync your system
declarch sync
```

## Related

- [`install` (command)](../commands/install.md) - Add packages to config
- [`sync`](sync.md) - Apply configuration to system
- [Remote Init Guide](../advanced/remote-init.md) - Advanced remote configurations
