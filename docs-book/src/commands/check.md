# check

Validate config and catch common mistakes.

## Usage

```bash
declarch check [COMMAND]
```

## Commands

| Command | Description |
|---------|-------------|
| `all` | full check (default) |
| `duplicates` | duplicated declarations |
| `conflicts` | same package name on multiple backends |
| `validate` | syntax/import only |

## Examples

```bash
declarch check
declarch check duplicates
declarch check conflicts
declarch check validate
```

## Useful flags

| Flag | Description |
|------|-------------|
| `--backend <NAME>` | scope by backend |
| `--diff` | show planning diff |
| `--fix` | apply supported fixes |
| `--benchmark` | timing info |
| `--modules <NAME>...` | temporary modules |
