# Command Overview

If you only memorize a few commands, memorize these.

## Core loop

```bash
declarch init
declarch install <backend:package...>
declarch sync
```

## Daily commands

- `declarch sync preview` - safe preview mode.
- `declarch search <query>` - find package candidates.
- `declarch info` - status summary.
- `declarch info list` - list all/orphans/synced views.
- `declarch explain <query>` - explain why a package appears in config/plan.
- `declarch lint` - quality checks for config files.

## Helpful sync variants

- `declarch sync update` - refresh indexes + sync.
- `declarch sync prune` - remove unmanaged packages.
- `declarch sync cache` - clean cache(s).
- `declarch sync upgrade` - run backend upgrades.
- `declarch sync sync --profile <name>` - opt-in profile layer.
- `declarch sync sync --host <name>` - opt-in host layer.

## Global flags

- `-y, --yes`
- `-f, --force`
- `-v, --verbose`
- `-q, --quiet`
- `--dry-run`

## Beginner advice

Use preview often and add backends gradually.
When using `install`, always specify backend via `backend:pkg` or `--backend`.
Keep base config portable; use `--profile`/`--host` only for extra machine-specific packages.
