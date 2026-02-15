# Troubleshooting (Advanced)

Use this page for concrete diagnosis when commands fail.

## 1) Backend not found

Error pattern:

```text
Backend 'xxx' not found
```

Actions:

```bash
declarch init --backend xxx
declarch check validate
```

Then confirm backend presence in `~/.config/declarch/backends.kdl` imports.

## 2) Missing backend binary

Error pattern:

```text
Package manager error: yarn not found
```

Actions:

1. install the binary, or
2. remove backend from active flow, or
3. configure fallback (`yarn -> npm`).

Verify with:

```bash
command -v yarn
```

## 3) Parse errors (KDL)

Error pattern includes line/column.

Actions:

- fix unbalanced braces
- check quoting for string values
- verify command templates contain required placeholders

Validate quickly:

```bash
declarch check validate
```

## 4) Search timeout/slow backend

Actions:

```bash
# narrow scope
declarch search firefox -b flatpak --limit 10

# local-only mode
declarch search firefox --local
```

Also audit backend `search`/`search_local` command for interactive behavior.

## 5) Sync appears to do nothing

Not always a failure. It often means desired and current state already match.

Inspect drift/orphans:

```bash
declarch info list orphans
```

## 6) Permissions / sudo

If backend requires root, ensure backend is configured correctly (`needs_sudo`) and your environment can prompt or run privileged commands.

Check config path permissions:

```bash
mkdir -p ~/.config/declarch
chmod 755 ~/.config/declarch
```

## 7) State reset procedure

```bash
rm ~/.config/declarch/state.json
declarch init
declarch sync preview
```

## 8) Debug bundle

Run this sequence before opening issue:

```bash
declarch -v check validate
declarch -v info
declarch -v sync preview
```

Issue tracker:
- https://github.com/nixval/declarch/issues
