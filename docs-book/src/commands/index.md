# Command Overview

If you only remember a few commands, remember these.

## Core workflow

```bash
declarch init
declarch install <package...>
declarch sync
```

## Daily commands

- `declarch sync preview`: see planned changes without execution.
- `declarch search <query>`: search package candidates.
- `declarch info`: show managed state summary.
- `declarch info list`: show managed package lists.

## Useful sync variants

- `declarch sync update`: refresh package indexes, then sync.
- `declarch sync prune`: sync and remove unmanaged packages.
- `declarch sync cache`: clean backend caches.
- `declarch sync upgrade`: upgrade packages through configured backends.

## Global flags

- `-y, --yes` skip confirmation prompts
- `-f, --force` force supported operations
- `-v, --verbose` detailed output
- `-q, --quiet` minimal output
- `--dry-run` preview mode

## Notes for beginners

- Start with `declarch sync preview` often.
- Add backends only when needed (`declarch init --backend <name>`).
- Keep configs small and modular.
