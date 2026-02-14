# sync

Synchronize system state with configuration.

## Usage

```bash
declarch sync [COMMAND] [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `sync` | Full sync (default behavior) |
| `preview` | Show changes without execution |
| `update` | Refresh indexes, then sync |
| `prune` | Remove unmanaged packages while syncing |
| `cache` | Clean backend cache(s) |
| `upgrade` | Upgrade packages through configured backends |

## Examples

```bash
# Standard sync
declarch sync

# Safe preview
declarch sync preview

# Update then sync
declarch sync update

# Sync and remove unmanaged packages
declarch sync prune

# Target only one package/backend
declarch sync sync --target firefox

# Enable hooks explicitly
declarch sync sync --hooks
```

## Common options

| Option | Description |
|--------|-------------|
| `--gc` | Garbage-collect orphans after sync |
| `--target <NAME>` | Sync only one package/scope |
| `--noconfirm` | Skip package manager prompts |
| `--hooks` | Enable lifecycle hooks |
| `--modules <NAME>...` | Temporarily include extra modules |
| `--diff` | Show plan diff (`sync/update/prune`) |

## Safety

Use preview first for large changes:

```bash
declarch sync preview
```
