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
1. Settings: `declarch settings set editor nvim`
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default)

## Examples

### Set Editor via Settings

```bash
declarch settings set editor nvim
```

### Set Editor via Environment

```bash
export EDITOR="code"
export VISUAL="code"
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
