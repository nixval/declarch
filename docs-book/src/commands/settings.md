# settings (removed)

`declarch settings` is no longer an active top-level command in v0.8.0.

## What to use instead

- Use global flags directly:

```bash
declarch --format json info
declarch -y sync
```

- Set preferred editor in config:

```kdl
editor "nvim"
```

- Use `declarch edit` for config changes:

```bash
declarch edit
declarch edit backends
```

If you still use old scripts with `declarch settings ...`, migrate them to flags/config.
