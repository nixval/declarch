# sync

Synchronize system with configuration.

## Usage

```bash
declarch sync [COMMAND] [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `sync` | Full sync (default) |
| `preview` | Preview changes without executing |
| `update` | Sync + update system packages |
| `prune` | Sync + remove unmanaged packages |

## Examples

### Basic Sync

```bash
declarch sync
```

Shows:
```
Changes:
  Adopt: (aur): neovim, bat, fzf
  Install: (npm): typescript

? Proceed with sync? [Y/n]
```

### Preview Changes

```bash
declarch sync preview
```

Shows what would happen without doing it.

### Sync with System Update

```bash
declarch sync update
```

Updates system packages first, then syncs.

### Remove Unmanaged Packages

```bash
declarch sync prune
```

Removes packages not in your config.

## Options

| Option | Description |
|--------|-------------|
| `--gc` | Garbage collect orphans after sync |
| `--target <NAME>` | Sync only specific package/backend |
| `--noconfirm` | Skip package manager prompts |
| `--hooks` | Enable lifecycle hooks |

## What "Adopt" Means

When you see:
```
Adopt: (aur): neovim
```

It means neovim is already installed. Declarch will "adopt" it into management.

## Safety

Sync always asks before making changes (unless `-y` flag).

```bash
# Auto-confirm
declarch sync -y
```
