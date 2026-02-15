# sync

Synchronize your system with config.

## Usage

```bash
declarch sync [COMMAND] [OPTIONS]
```

## Commands

| Command | Description |
|---------|-------------|
| `sync` | normal sync |
| `preview` | show plan only |
| `update` | refresh indexes then sync |
| `prune` | remove unmanaged packages |
| `cache` | clean backend cache |
| `upgrade` | run backend upgrades |

## Typical flow

```bash
declarch sync preview
declarch sync
```

## More examples

```bash
declarch sync update
declarch sync prune
declarch sync sync --target firefox
declarch sync sync --hooks
```

## Common options

| Option | Description |
|--------|-------------|
| `--gc` | garbage-collect orphans after sync |
| `--target <NAME>` | sync one package/scope |
| `--noconfirm` | skip backend prompt flags |
| `--hooks` | enable lifecycle hooks |
| `--modules <NAME>...` | temporary extra modules |
| `--diff` | show plan diff |
