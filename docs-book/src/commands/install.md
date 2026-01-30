# declarch install

Add packages to your configuration.

## Usage

```bash
declarch install <PACKAGE>... [OPTIONS]
```

## Options

- `<PACKAGE>...` - Package(s) to install (format: `[backend:]package`)
- `--backend <BACKEND>` / `-b` - Target package manager for all packages
- `--module <NAME>` / `-m` - Add to specific module instead of `others`
- `--no-sync` - Don't sync after editing config
- `--yes` / `-y` - Skip confirmation prompts (global flag)
- `--dry-run` - Preview changes without executing (global flag)

## Quick Start

```bash
# Install packages (adds to default module)
declarch install bat fzf ripgrep

# Install to specific module
declarch install neovim --module dev

# Install from specific backend
declarch install soar:ripgrep

# Skip confirmation
declarch install bat -y

# Preview what would be installed
declarch install bat --dry-run
```

## What It Does

1. Adds packages to your configuration file
2. Installs the packages via sync
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

# All packages from npm
declarch install typescript eslint --backend npm
```

### Multiple Packages

```bash
declarch install bat exa ripgrep fzf
```

### Skip Confirmation

```bash
declarch install bat -y
```

Useful for scripts and automation.

### Dry Run

```bash
declarch install bat --dry-run
```

Output:
```
Dry Run: Installing Packages
Would install the following packages:
  • bat → module: others
Would sync after installation
```

### Add Without Syncing

```bash
declarch install bat --no-sync
```

Adds to config but doesn't install. Useful when editing multiple packages before syncing.

## Package Format

Packages can be specified in several ways:

```bash
# Simple package (uses default backend)
declarch install bat

# Explicit backend
declarch install aur:bat

# Mix formats
declarch install bat npm:eslint soar:ripgrep
```

## Supported Backends

| Backend | Description | Example |
|---------|-------------|---------|
| `aur` | Arch User Repository (default) | `declarch install bat` |
| `soar` | Cross-distro static binaries | `declarch install soar:bat` |
| `flatpak` | Flatpak applications | `declarch install flatpak:com.spotify.Client` |
| `npm` | Node.js packages | `declarch install npm:eslint` |
| `yarn` | Yarn packages | `declarch install yarn:eslint` |
| `pnpm` | PNPM packages | `declarch install pnpm:eslint` |
| `bun` | Bun packages | `declarch install bun:eslint` |
| `pip` | Python packages | `declarch install pip:requests` |
| `cargo` | Rust packages | `declarch install cargo:ripgrep` |
| `brew` | Homebrew packages | `declarch install brew:bat` |

## Automatic Rollback

If installation fails (e.g., package not found), declarch automatically:

1. Creates a timestamped backup before modifying KDL files
2. Restores from backup on sync failure
3. Cleans up backups on successful install

No manual cleanup needed!

## Module Selection

```bash
# Default: adds to modules/others.kdl
declarch install bat

# Add to specific module
declarch install bat --module base

# Add to nested module
declarch install bat --module desktop/hyprland
```

## Common Workflow

```bash
# Add packages (auto-syncs)
declarch install bat fzf ripgrep

# They're now in modules/others.kdl
cat modules/others.kdl

# Already synced! No need to run declarch sync

# Add multiple times before syncing
declarch install bat --no-sync
declarch install fzf --no-sync
declarch install ripgrep --no-sync
declarch sync
```

## Automation

```bash
# Script-friendly usage
declarch install bat -y --no-sync
declarch sync -y
```

## Related

- [`sync`](sync.md) - Sync all packages
- [`edit`](edit.md) - Edit configuration manually
- [`check`](check.md) - Validate configuration
- [`info`](info.md) - View installed packages
