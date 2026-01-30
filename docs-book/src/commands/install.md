# declarch install

Add packages to your configuration.

## Usage

```bash
declarch install <PACKAGE>... [OPTIONS]
```

## Quick Start

```bash
# Install packages (adds to default module)
declarch install bat fzf ripgrep

# Install to specific module
declarch install neovim --module dev

# Install from specific backend
declarch install soar:ripgrep
```

## What It Does

1. Adds packages to your configuration file
2. Installs the packages
3. Tracks them in declarch state

**If installation fails**, automatically restores your config files (automatic rollback).

## Examples

### Basic Installation

```bash
declarch install bat exa ripgrep
```

Adds packages to `modules/others.kdl` and installs them.

### Install to Module

```bash
declarch install python nodejs --module dev
```

Adds packages to `modules/dev.kdl`.

### Install from Specific Backend

```bash
# From Soar instead of AUR
declarch install soar:bat

# From Flatpak
declarch install flatpak:com.spotify.Client
```

### Multiple Packages

```bash
declarch install bat exa ripgrep fzf fd
```

## Options

| Option | Description |
|--------|-------------|
| `--backend <BACKEND>` | Install from specific backend (aur, flatpak, npm, etc.) |
| `--module <NAME>` | Add to specific module instead of `others` |
| `-y, --yes` | Skip confirmation prompts |

## Supported Backends

| Backend | Description |
|---------|-------------|
| `aur` | Arch User Repository (default) |
| `soar` | Cross-distro packages |
| `flatpak` | Flatpak applications |
| `npm` | Node.js packages |
| `yarn` | Yarn packages |
| `pnpm` | PNPM packages |
| `bun` | Bun packages |
| `pip` | Python packages |
| `cargo` | Rust packages |
| `brew` | Homebrew packages |

## Automatic Rollback

If installation fails (e.g., package not found), declarch automatically restores your config files from backup. No manual cleanup needed.

## Common Workflow

```bash
# Add packages
declarch install bat fzf ripgrep

# They're now in modules/others.kdl
cat modules/others.kdl

# Already synced! No need to run declarch sync
```

## Related

- [`sync`](sync.md) - Sync all packages
- [`edit`](edit.md) - Edit configuration manually
- [`check`](check.md) - Validate configuration
