# info

Show status, diagnosis, and package reasoning in one place.

## Usage

```bash
declarch info [QUERY] [FLAGS]
```

## Common examples

```bash
# status (default)
declarch info

# doctor
declarch info --doctor

# list views
declarch info --list
declarch info --list --orphans
declarch info --list --synced

# reasoning (replaces old explain)
declarch info bat
declarch info aur:bat
declarch info system/base

declarch info --plan
```

## Useful flags

- `--doctor`: run diagnostics
- `--plan`: show sync install/remove drift reasoning
- `--list`: list managed packages
- `--orphans`: with `--list`, show orphan packages only
- `--synced`: with `--list`, show synced packages only
- `--backend <name>`: filter status/list output by backend
- `--package <name>`: filter status output by package name
- `--profile`, `--host`, `--modules`: apply optional context for reasoning mode

## Machine output (v1)

For integrations/scripts, you can request contract envelope output:

```bash
declarch info --format json --output-version v1
declarch info --list --format yaml --output-version v1
```

Use this for scripts, CI, and integrations that need stable structured output.

## Notes

- Use one mode per call: status, query, `--plan`, `--doctor`, or list mode.
- If a backend is not meant for current OS, checks can skip it gracefully.
