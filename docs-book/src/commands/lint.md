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
```

## Flags

- `--mode all|validate|duplicates|conflicts`: lint scope
- `--backend <name>`: backend filter for package-level checks
- `--diff`: show planned install/remove drift
- `--fix`: apply safe automatic fixes
- `--strict`: warnings become blocking errors
- `--benchmark`: show elapsed time
- `--profile`, `--host`, `--modules`: include optional overlays/modules

## Recommended flow

```bash
declarch lint
declarch lint --fix
declarch lint --strict
```
