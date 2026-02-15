# info

Show current state and diagnostics.

## Usage

```bash
declarch info [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `status` | State summary (default) |
| `list` | Package list views |
| `doctor` | Diagnose common issues |

## Examples

```bash
# Status summary
declarch info

# List managed packages
declarch info list

# List orphan packages
declarch info list orphans

# Diagnostics
declarch info doctor
```

## `info list` views

```bash
declarch info list all
declarch info list orphans
declarch info list synced
```

Optional backend filter:

```bash
declarch info list orphans --backend aur
```

## Common filters

```bash
declarch info status --backend aur
declarch info doctor --package firefox
```
