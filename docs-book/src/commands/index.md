# Command Overview

If you only remember a few commands, remember these.

## Core workflow

```bash
declarch init
declarch install <package...>
declarch sync
```

## Daily commands

- `declarch sync preview`:
  see planned changes, no execution.
- `declarch search <query>`:
  search package candidates.
- `declarch info`:
  check current managed state.
- `declarch list`:
  list managed/orphan/synced groups.

## Useful sync variants

- `declarch sync update`:
  update backend indexes + sync.
- `declarch sync prune`:
  remove unmanaged packages.

## Global flags

- `-y, --yes` skip confirmations
- `-f, --force` force overwrite/operations where supported
- `-v, --verbose` detailed output
- `-q, --quiet` minimal output

## Notes for beginners

- Start with `preview` often.
- Add backends only when needed (`declarch init --backend <name>`).
- Keep configs small and modular.
