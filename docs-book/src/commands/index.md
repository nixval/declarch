# Commands

## Essential Commands

| Command | Description |
|---------|-------------|
| [`init`](init.md) | Create initial configuration |
| [`install`](install.md) | Add packages to config |
| [`sync`](sync.md) | Apply config to system |

## Management Commands

| Command | Description |
|---------|-------------|
| [`sync preview`](sync.md#preview) | Preview changes |
| [`sync update`](sync.md#update) | Sync + update system |
| [`sync prune`](sync.md#prune) | Remove unmanaged packages |
| [`check`](check.md) | Verify config |
| [`info`](info.md) | Show system status |
| [`list`](list.md) | List packages |

## Utility Commands

| Command | Description |
|---------|-------------|
| [`search`](search.md) | Search for packages |
| [`edit`](edit.md) | Edit config files |
| [`switch`](switch.md) | Switch package variants |
| [`settings`](settings.md) | Manage settings |

## Global Flags

These work with any command:

| Flag | Description |
|------|-------------|
| `-y, --yes` | Skip confirmations |
| `-v, --verbose` | More output |
| `-q, --quiet` | Less output |
| `-f, --force` | Force operations |
| `--dry-run` | Preview only |
| `--format <fmt>` | Output format (table, json, yaml) |

## Quick Reference

```bash
# Initialize
declarch init

# Add packages
declarch install neovim bat fzf

# Sync
declarch sync

# Preview changes
declarch sync preview

# Search
declarch search firefox
```
