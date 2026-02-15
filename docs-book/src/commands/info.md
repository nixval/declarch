# info

Show status and diagnostics.

## Usage

```bash
declarch info [COMMAND]
```

## Commands

| Command | Description |
|---------|-------------|
| `status` | summary (default) |
| `list` | package list views |
| `doctor` | diagnostics |

## Examples

```bash
declarch info
declarch info list
declarch info list orphans
declarch info list synced
declarch info doctor
```

## Filters

```bash
declarch info status --backend aur
declarch info doctor --package firefox
```
