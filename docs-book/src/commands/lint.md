# lint

Lint checks config quality with simple, beginner-friendly rules.

## Usage

```bash
declarch lint
```

## Examples

```bash
declarch lint
declarch lint --strict
declarch lint --fix
declarch lint --profile desktop --host vps-1
```

## Current checks

- duplicate package declarations
- cross-backend same-name conflicts
- legacy `packages` syntax usage
- unresolved `imports` paths
- hooks configured without `experimental { "enable-hooks" }`
- implicit `default` backend from legacy syntax
- policy-aware severity (`on_duplicate`, `on_conflict`, `require_backend`)

## Flags

- `--strict`: fail on warnings
- `--fix`: apply safe fixes (currently import sorting/cleanup)
- `--profile`, `--host`: include optional overlays
- `--modules`: include extra modules for this lint run
