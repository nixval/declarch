# info list

List managed package views.

## Usage

```bash
declarch info list [COMMAND] [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `all` | All managed packages (default) |
| `orphans` | Installed but unmanaged packages |
| `synced` | Packages both declared and installed |

## Examples

```bash
# Default list (all)
declarch info list

# Orphans
declarch info list orphans

# Synced only
declarch info list synced

# Filter by backend
declarch info list orphans --backend aur
```

## Option

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | Filter by backend |

## Compatibility note

Older docs/examples may show `declarch list ...`.
Current command path is `declarch info list ...`.
