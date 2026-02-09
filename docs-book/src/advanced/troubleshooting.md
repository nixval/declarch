# Troubleshooting

## Common Issues

### "Backend 'xxx' not found"

**Cause:** Backend not installed or not imported.

**Fix:**
```bash
# Install the backend
declarch init --backend xxx

# Or re-initialize
declarch init
```

### "Failed to parse KDL document"

**Cause:** Syntax error in your .kdl file.

**Fix:**
Check the error message - it shows line and column:

```
error: No closing '}' for child block
  --> ~/.config/declarch/base.kdl:8:5
   |
 8 │     meta {
   │      ^
```

Common fixes:
- Add quotes: `format "whitespace"` not `format whitespace`
- Close braces: Every `{` needs a `}`
- Quote booleans: `needs_sudo "true"` not `needs_sudo true`

### Sync Shows No Changes

**Cause:** Packages already installed.

**Fix:** This is normal. Use `declarch list orphans` to see unmanaged packages.

### Permission Denied

**Cause:** Declarch needs write access to `~/.config/declarch/`.

**Fix:**
```bash
mkdir -p ~/.config/declarch
chmod 755 ~/.config/declarch
```

## Debug Mode

Get more output:

```bash
declarch -v sync
declarch -vv sync  # Even more verbose
```

## Check Config

Validate without syncing:

```bash
declarch check validate
```

## Reset State

If state gets corrupted:

```bash
rm ~/.config/declarch/state.json
declarch init
```

## Still Stuck?

1. Run `declarch check` for diagnostics
2. Run with `-v` for verbose output
3. Check [GitHub Issues](https://github.com/nixval/declarch/issues)
