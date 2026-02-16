# sync

Synchronize your system with config.

## Usage

```bash
declarch sync [COMMAND] [OPTIONS]
```

## Commands

| Command | Description |
|---------|-------------|
| `(default)` | normal sync |
| `update` | refresh indexes then sync |
| `prune` | remove unmanaged packages |
| `cache` | clean backend cache |
| `upgrade` | run backend upgrades |

## Typical flow

```bash
declarch --dry-run sync
declarch sync
```

## More examples

```bash
declarch sync update
declarch sync prune
declarch sync --target firefox
declarch sync --hooks
declarch sync --profile desktop
declarch sync --host vps-1
```

## Common options

| Option | Description |
|--------|-------------|
| `--target <NAME>` | sync one package/scope |
| `--profile <NAME>` | activate `profile "NAME" { ... }` block |
| `--host <NAME>` | activate `host "NAME" { ... }` block |
| `--noconfirm` | skip backend prompt flags |
| `--hooks` | enable lifecycle hooks |
| `--modules <NAME>...` | temporary extra modules |
| `--diff` | show plan diff |

## Machine output (v1)

```bash
declarch --dry-run sync --format json --output-version v1
declarch --dry-run sync --format yaml --output-version v1
```

This emits one machine envelope summary for integrations.

## Hook safety gate

Even with `--hooks`, hooks are blocked unless you explicitly opt in from config:

```kdl
experimental {
    "enable-hooks"
}
```

Without that block, `declarch` shows hook entries but does not execute them.

## Cross-machine behavior

If you share one config across Linux/macOS/Windows in the future, some backends may not fit every OS.
Declarch will skip incompatible backends and continue the sync flow.
The same skip behavior is used by related flows like `sync cache` and `sync upgrade`.
