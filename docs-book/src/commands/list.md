# list

List packages.

## Usage

```bash
declarch list [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `all` | All packages (default) |
| `orphans` | Unmanaged packages |
| `synced` | Successfully synced packages |

## Examples

### List All

```bash
declarch list
```

### List Orphans

```bash
declarch list orphans
```

Shows packages installed on system but not in config.

### Filter by Backend

```bash
declarch list --backend aur
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | Filter by backend |
