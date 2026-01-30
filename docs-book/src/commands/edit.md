# declarch edit

Edit configuration files in your editor.

## Usage

```bash
declarch edit [TARGET]
```

## Quick Start

```bash
# Edit main config
declarch edit

# Edit a module
declarch edit base

# Edit nested module
declarch edit dev/rust
```

## How It Works

The editor is chosen in this order:
1. `editor "name"` in your `declarch.kdl`
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default)

## Examples

### Set Editor in Config

```kdl
// In declarch.kdl
editor "nvim"
```

### Set Editor via Environment

```bash
export EDITOR="code"
export VISUAL="code"
```

### Common Editors

```kdl
editor "nvim"     // Neovim
editor "vim"      // Vim
editor "code"     // VS Code (use --wait for GUI)
editor "hx"       // Helix
```

## What It Opens

| Target | File |
|--------|------|
| (none) | `~/.config/declarch/declarch.kdl` |
| `base` | `~/.config/declarch/modules/base.kdl` |
| `dev/rust` | `~/.config/declarch/modules/dev/rust.kdl` |

## Common Workflow

```bash
# Edit and validate
declarch edit && declarch check

# Edit, check, and sync
declarch edit && declarch check && declarch sync
```

## Related

- [`check`](check.md) - Validate after editing
- [`sync`](sync.md) - Apply changes
- [KDL Syntax Guide](../configuration/kdl-syntax.md)
