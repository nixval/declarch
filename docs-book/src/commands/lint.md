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
```

## Flags

- `--mode all|validate|duplicates|conflicts`: lint scope
- `--backend <name>`: backend filter for package-level checks
- `--diff`: show planned install/remove drift
- `--fix`: apply safe automatic fixes
- `--strict`: warnings become blocking errors
- `--benchmark`: show elapsed time
- `--repair-state`: sanitize broken state entries (no manual JSON edits)
- `--profile`, `--host`, `--modules`: include optional overlays/modules

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
