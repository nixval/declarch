# lint

Lint checks config quality with simple, beginner-friendly rules.

## Why use it

- catch mistakes before running `sync`
- keep shared configs tidy and predictable
- spot old syntax that still works but should be migrated

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
declarch lint --modules system/base
```

## Current checks

- duplicate package declarations
- cross-backend same-name conflicts
- legacy `packages` syntax usage
- unresolved `imports` paths
- unresolved `backend_imports` paths
- hooks configured without `experimental { "enable-hooks" }`
- implicit `default` backend from legacy syntax
- policy-aware severity (`on_duplicate`, `on_conflict`, `require_backend`)
- recursively checks imported files, not only root file

## Flags

- `--strict`: fail on warnings
- `--fix`: apply safe fixes (currently import sorting/cleanup)
- `--profile`, `--host`: include optional overlays
- `--modules`: include extra modules for this lint run

## Recommended flow

```bash
# 1) Check
declarch lint

# 2) Apply safe autofix
declarch lint --fix

# 3) Re-check, strict for CI
declarch lint --strict
```

## Exit behavior

- exit `0`: no errors (and no warnings in `--strict`)
- exit non-zero: lint found blocking issues
