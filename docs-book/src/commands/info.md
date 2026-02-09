# info

Show system status.

## Usage

```bash
declarch info [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `status` | Show status (default) |
| `doctor` | Diagnose issues |

## Examples

### Show Status

```bash
declarch info
```

Shows:
- Total managed packages
- Packages per backend
- Sync status

### Diagnose Issues

```bash
declarch info doctor
```

Runs diagnostic checks:
- Config validity
- Backend availability
- Missing dependencies

## Options

| Option | Description |
|--------|-------------|
| `--backend <NAME>` | Filter by backend |
| `--package <NAME>` | Filter by package |
