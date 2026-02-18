# Troubleshooting (Advanced)

Use this page when a command fails or behaves differently than expected.
Start with simple checks first, then move to deeper checks.

## Quick first checks

```bash
declarch lint --mode validate
declarch info --doctor
declarch --dry-run sync
```

## 1) Backend not found

Error pattern:

```text
Backend 'xxx' not found
```

Actions:

```bash
declarch init --backend xxx
declarch lint --mode validate
```

Then confirm backend import in `declarch.kdl` under `backends { ... }`.
If you are not sure where your config lives on this OS, run:

```bash
declarch info --doctor
```

## 2) Missing backend binary

Error pattern:

```text
Package manager error: yarn not found
```

Actions:

1. install the binary, or
2. stop using that backend for now, or
3. set a compatible fallback backend (example: `nala -> apt`).

Verify with:

```bash
declarch info yarn -v
```

## 3) Parse errors (KDL)

Error pattern includes line/column.

Actions:

- fix unbalanced braces
- check quote usage in strings
- verify backend command templates include required placeholders (`{packages}`, `{query}`, `{binary}` when needed)

Validate quickly:

```bash
declarch lint --mode validate
```

## 4) Search timeout/slow backend

Actions:

```bash
# narrow scope
declarch search firefox -b flatpak --limit 10

# local-only mode
declarch search firefox --local
```

If this keeps happening, check backend `search`/`search_local` commands and avoid interactive prompts.

## 5) Sync appears to do nothing

This is often normal: desired state already matches installed state.

Inspect drift/orphans:

```bash
declarch info --list --scope orphans
```

## 6) Permissions / sudo

If backend requires root, ensure backend is configured correctly (`needs_sudo`) and your environment can prompt or run privileged commands.

Linux path permission check example:

```bash
mkdir -p ~/.config/declarch
chmod 755 ~/.config/declarch
```

On macOS/Windows, use:

```bash
declarch info --doctor
```

## 7) State reset procedure

If state is corrupted or stale, reset and re-check:

```bash
rm ~/.local/state/declarch/state.json
declarch init
declarch --dry-run sync
```

For non-Linux paths, first find your real state path with:

```bash
declarch info --doctor
```

## 8) Debug bundle

Before opening an issue, collect this output:

```bash
declarch -v lint --mode validate
declarch -v info --doctor
declarch -v --dry-run sync
```

Issue tracker:
- https://github.com/nixval/declarch/issues
