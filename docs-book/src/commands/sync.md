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
declarch sync sync --profile desktop
declarch sync sync --host vps-1
```

## Common options

| Option | Description |
|--------|-------------|
| `--gc` | garbage-collect orphans after sync |
| `--target <NAME>` | sync one package/scope |
| `--profile <NAME>` | activate `profile "NAME" { ... }` block |
| `--host <NAME>` | activate `host "NAME" { ... }` block |
| `--noconfirm` | skip backend prompt flags |
| `--hooks` | enable lifecycle hooks |
| `--modules <NAME>...` | temporary extra modules |
| `--diff` | show plan diff |

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
