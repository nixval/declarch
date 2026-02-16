# Debugging

This page is a practical troubleshooting playbook for config, state, and backend behavior.

## Quick triage

Run these first:

```bash
declarch info --doctor
declarch info --list --scope all
declarch lint
declarch --dry-run sync
```

What this gives you:

- `info --doctor`: runtime/config/state health checks
- `info --list --scope all`: everything tracked in state
- `lint`: config and consistency warnings
- `--dry-run sync`: planned install/adopt/prune actions before mutation

## Lint for debugging (soft checks)

Use lint modes to isolate problem type quickly:

```bash
# syntax/import/basic structure
declarch lint --mode validate

# duplicate declarations
declarch lint --mode duplicates

# known config conflicts
declarch lint --mode conflicts

# inspect planned drift from current state
declarch lint --diff

# safe automatic cleanup
declarch lint --fix

# strict CI-like mode (warnings fail)
declarch lint --strict

# state structure repair only
declarch lint --repair-state
```

Helpful filters:

```bash
declarch lint --mode conflicts --backend aur
declarch lint --mode duplicates --backend flatpak
```

## Info for debugging

Use `info` to explain state and scope:

```bash
# full tracked state
declarch info --list --scope all

# only unmanaged installed packages
declarch info --list --scope unmanaged

# backend focus
declarch info --list --scope all --backend soar

# package-focused status/reasoning
declarch info firefox
declarch info --package firefox
```

## State mismatch (safe fix, no uninstall)

Use when `sync` shows unexpected `Adopt`, wrong backend mapping, or stale entries.

### 1) Preview

```bash
declarch lint --state-rm soar:firefox --dry-run
```

### 2) Remove from state

```bash
declarch lint --state-rm soar:firefox
```

### 3) Verify

```bash
declarch info --list --scope all --backend soar
declarch --dry-run sync
```

Common state remove patterns:

```bash
# exact id
declarch lint --state-rm backend:package

# plain name in one backend
declarch lint --state-rm package --state-rm-backend backend

# remove all entries for one backend
declarch lint --state-rm-backend backend --state-rm-all
```

## Search debugging tips

```bash
# backend-specific search
declarch search soar:firefox
declarch search firefox --backends soar

# local installed scan only
declarch search firefox --local

# managed-only check (declarch state)
declarch search firefox --installed-only
```

Use `--verbose` when you need backend timing/error details.

## Suggested workflow

```bash
declarch info --doctor
declarch lint --mode validate
declarch lint --diff
declarch --dry-run sync
```

If the plan still looks wrong, use targeted state cleanup (`--state-rm*`) and re-run dry-run sync.

## Notes

- `--state-rm*` only changes declarch state; it does **not** uninstall packages.
- If package name is ambiguous across backends, use `backend:package`.
- Add `-y` for non-interactive runs.
