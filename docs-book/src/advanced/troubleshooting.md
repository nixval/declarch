# Troubleshooting

## Common issues

### "Backend 'xxx' not found"

Cause: backend definition is not available locally.

Fix:

```bash
declarch init --backend xxx
```

### "Failed to parse KDL document"

Cause: syntax error in config.

Fix: follow line/column from error output.

Common fixes:
- close every `{` with `}`
- quote string fields when required
- for booleans, use consistent style (`true`/`false` or quoted form)

### Search warnings for missing binary

Cause: backend config exists, but binary is not installed (example: `yarn not found`).

Fix options:
- install the binary, or
- remove that backend from active use, or
- set backend fallback (example `yarn -> npm`).

### Search hangs or timeout on slow backend

Cause: backend search command is slow or interactive.

Fix:
- limit target backend: `declarch search firefox -b flatpak`
- reduce scope using `--limit`
- verify backend search command in backend file.

### "No changes" during sync

Cause: desired state already matches system.

This is normal. Check coverage with:

```bash
declarch info list orphans
```

### Permission denied

Cause: no write access to `~/.config/declarch/`.

Fix:

```bash
mkdir -p ~/.config/declarch
chmod 755 ~/.config/declarch
```

## Debug mode

```bash
declarch -v sync
declarch -v check
```

## Validate config only

```bash
declarch check validate
```

## Reset state

```bash
rm ~/.config/declarch/state.json
declarch init
```

## Still stuck

1. Run `declarch check`.
2. Run with `-v`.
3. Open issue: https://github.com/nixval/declarch/issues
