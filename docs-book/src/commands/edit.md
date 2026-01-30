# declarch edit

Edit configuration files in your editor.

## Usage

```bash
declarch edit [TARGET] [OPTIONS]
```

## Options

- `[TARGET]` - Module or config to edit (optional)
- `--dry-run` - Preview which file would be opened without opening it

## Quick Start

```bash
# Edit main config
declarch edit

# Edit a module
declarch edit base

# Preview what would be edited
declarch edit base --dry-run

# Edit nested module
declarch edit dev/rust
```

## How It Works

The editor is chosen in this order:
1. Settings: `declarch settings set editor nvim`
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default)

## Examples

### Edit Root Config

```bash
declarch edit
```

Opens `~/.config/declarch/declarch.kdl`.

### Edit Module

```bash
declarch edit base
```

Opens `~/.config/declarch/modules/base.kdl`.

### Edit Nested Module

```bash
declarch edit dev/rust
```

Opens `~/.config/declarch/modules/dev/rust.kdl`.

### Dry Run

```bash
declarch edit base --dry-run
```

Output:
```
Dry Run: Edit Configuration
Would open: /home/user/.config/declarch/modules/base.kdl
With editor: nvim
```

Useful to see which file will be edited before opening.

### Set Editor via Settings

```bash
declarch settings set editor nvim
declarch edit
```

### Set Editor via Environment

```bash
export EDITOR="code"
export VISUAL="code"
declarch edit
```

### Common Editors

```bash
# Neovim
declarch settings set editor nvim

# VS Code
declarch settings set editor "code --wait"

# Vim
declarch settings set editor vim

# Helix
declarch settings set editor hx
```

Note: GUI editors like VS Code need the `--wait` flag to work properly.

## What It Opens

| Target | File |
|--------|------|
| (none) | `~/.config/declarch/declarch.kdl` |
| `base` | `~/.config/declarch/modules/base.kdl` |
| `dev/rust` | `~/.config/declarch/modules/dev/rust.kdl` |
| `gaming` | `~/.config/declarch/modules/gaming.kdl` |

## Module Resolution

1. **Exact match**: `declarch edit base` → `modules/base.kdl`
2. **Nested path**: `declarch edit dev/rust` → `modules/dev/rust.kdl`
3. **Root config**: `declarch edit` (no target) → `declarch.kdl`

## Common Workflow

```bash
# Edit and validate
declarch edit && declarch check

# Edit, check, and sync
declarch edit && declarch check && declarch sync

# Preview before editing
declarch edit base --dry-run

# Edit specific module
declarch edit base
declarch check --backend aur
declarch sync --modules base
```

## Tips

### Quick Edit Cycle

```bash
# Edit, validate, sync in one command
declarch edit && declarch check -v && declarch sync
```

### Edit Multiple Modules

```bash
# Edit root config
declarch edit

# Edit specific module
declarch edit base

# Verify before syncing
declarch check
```

### Use with Dry Run

```bash
# Check which file before opening
declarch edit dev/rust --dry-run
```

## Related

- [`check`](check.md) - Validate after editing
- [`sync`](sync.md) - Apply changes
- [`install`](install.md) - Add packages interactively
- [KDL Syntax Guide](../configuration/kdl-syntax.md)
