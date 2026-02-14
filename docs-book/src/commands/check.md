# check

Validate configuration and consistency.

## Usage

```bash
declarch check [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `all` | Run all checks (default) |
| `duplicates` | Find duplicate declarations |
| `conflicts` | Find cross-backend name conflicts |
| `validate` | Syntax/import validation only |

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
| `--backend <NAME>` | Filter backend (where supported) |
| `--diff` | Show planned diff output |
| `--fix` | Auto-fix where possible |
| `--benchmark` | Show perf metrics (`all`/`validate`) |
| `--modules <NAME>...` | Temporarily include extra modules |
