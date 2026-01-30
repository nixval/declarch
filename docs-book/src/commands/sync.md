# declarch sync

Synchronize your system with the configuration.

## Usage

```bash
declarch sync [OPTIONS]
```

## Quick Start

```bash
# Basic sync
declarch sync

# Sync with system update
declarch sync -u

# Preview changes first
declarch sync --dry-run
```

## What It Does

1. Reads your configuration
2. Installs missing packages
3. Tracks existing packages
4. Optionally removes unwanted packages (`--prune`)

## Common Options

| Option | Description |
|--------|-------------|
| `-u, --update` | Update system first (like `paru -Syu`) |
| `--dry-run` | Preview changes without applying |
| `--prune` | Remove packages not in config |
| `--target <BACKEND>` | Sync only specific backend (e.g., `aur`, `flatpak`) |

## Examples

### Daily Update

```bash
declarch sync -u
```

Updates system and syncs packages.

### Remove Unwanted Packages

```bash
# First, preview what will be removed
declarch sync --prune --dry-run

# If satisfied, apply
declarch sync --prune
```

### Sync Specific Backend

```bash
# Only AUR packages
declarch sync --target aur

# Only Flatpak apps
declarch sync --target flatpak
```

### Automated/Script Usage

```bash
declarch sync --noconfirm --hooks
```

Skips prompts and runs hooks (for CI/CD).

## Safety Tips

1. **Always dry-run before pruning:**
   ```bash
   declarch sync --prune --dry-run
   ```

2. **Check configuration first:**
   ```bash
   declarch check && declarch sync
   ```

## Related

- [`check`](check.md) - Validate configuration
- [`info`](info.md) - View system status
- [`install`](install.md) - Add packages to config
