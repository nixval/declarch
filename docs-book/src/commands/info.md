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
| `list` | List views (`all`, `orphans`, `synced`) |
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

## Common filters

```bash
declarch info status --backend aur
declarch info doctor --package firefox
```
