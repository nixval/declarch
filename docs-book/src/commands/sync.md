# declarch sync

Synchronize your system with the configuration.

## Usage

```bash
declarch sync [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `sync` - Full sync (default)
- `preview` - Preview changes without executing
- `update` - Sync with system update
- `prune` - Sync with package removal

## Options

- `--target <BACKEND>` - Sync only specific backend (e.g., `aur`, `flatpak`)
- `--noconfirm` - Skip package manager confirmation prompts
- `--hooks` - Enable hooks (disabled by default for security)
- `--skip-soar-install` - Skip automatic Soar installation
- `--modules <MODULES>` - Load additional modules temporarily
- `--gc` - Garbage collect system orphans after sync

## Quick Start

```bash
# Basic sync
declarch sync

# Sync with system update
declarch sync update

# Preview changes first
declarch sync preview

# Remove unwanted packages
declarch sync prune
```

## What It Does

1. Reads your configuration
2. Installs missing packages
3. Tracks existing packages
4. Optionally removes unwanted packages (with `prune` subcommand)

## Examples

### Daily Update

```bash
declarch sync update
```

Updates system and syncs packages.

### Preview Changes

```bash
declarch sync preview
```

Shows what would be installed, updated, or removed without making changes.

### Remove Unwanted Packages

```bash
# First, preview what will be removed
declarch sync prune --target aur

# If satisfied, apply
declarch sync prune
```

### Sync Specific Backend

```bash
# Only AUR packages
declarch sync --target aur

# Only Flatpak apps
declarch sync --target flatpak
```

### Update with Cleanup

```bash
declarch sync update --gc
```

Updates system, syncs packages, and removes orphan packages.

### Automated/Script Usage

```bash
declarch sync --noconfirm --hooks
```

Skips prompts and runs hooks (for CI/CD).

## Safety Tips

1. **Always preview before pruning:**
   ```bash
   declarch sync prune --target aur --dry-run
   ```

2. **Check configuration first:**
   ```bash
   declarch check && declarch sync
   ```

## Migration from Flags

Old flag-based syntax is deprecated but still works:

```bash
# Old (deprecated, will be removed in v0.7.0)
declarch sync --dry-run
declarch sync --update
declarch sync --prune

# New (recommended)
declarch sync preview
declarch sync update
declarch sync prune
```

## Related

- [`check`](check.md) - Validate configuration
- [`info`](info.md) - View system status
- [`install`](install.md) - Add packages to config

