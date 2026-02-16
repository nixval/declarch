# lint

Lint checks configuration quality before sync.

## Usage

```bash
declarch lint [FLAGS]
```

## Common examples

```bash
# full checks
declarch lint

# syntax/import checks only (replaces old check validate)
declarch lint --mode validate

# focused checks
declarch lint --mode duplicates
declarch lint --mode conflicts --backend aur

# optional extras
declarch lint --diff
declarch lint --fix
declarch lint --strict
declarch lint --benchmark
declarch lint --repair-state

# state cleanup (state only, no uninstall)
declarch lint --state-rm soar:firefox --dry-run
declarch lint --state-rm soar:firefox
declarch lint --state-rm package --state-rm-backend soar
declarch lint --state-rm-backend soar --state-rm-all
```

## Flags

- `--mode all|validate|duplicates|conflicts`: lint scope
- `--backend <name>`: backend filter for package-level checks
- `--diff`: show planned install/remove drift
- `--fix`: apply safe automatic fixes
- `--strict`: warnings become blocking errors
- `--benchmark`: show elapsed time
- `--repair-state`: sanitize broken state entries (no manual JSON edits)
- `--state-rm <ids>`: remove state entries by `backend:package` or plain package name
- `--state-rm-backend <name>`: backend scope for plain names, or for backend-wide cleanup
- `--state-rm-all`: remove all tracked entries for `--state-rm-backend`
- `--profile`, `--host`, `--modules`: include optional overlays/modules

## Minimal playbook (recommended)

```bash
declarch lint --state-rm backend:package --dry-run
declarch lint --state-rm backend:package
declarch --dry-run sync
```

For more debugging flows: [Advanced Debugging](../advanced/debugging.md).

## Machine output (v1)

```bash
declarch lint --format json --output-version v1
declarch lint --format yaml --output-version v1
```

When this mode is used, lint prints structured envelope output for automation/integrations.

## Recommended flow

```bash
declarch lint
declarch lint --fix
declarch lint --strict
```
