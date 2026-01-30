# declarch check

Validate your configuration.

## Usage

```bash
declarch check [OPTIONS]
```

## Quick Start

```bash
# Basic check
declarch check

# List all packages
declarch check --verbose

# Find duplicates
declarch check --duplicates
```

## What It Checks

1. Configuration file exists
2. KDL syntax is valid
3. All imports resolve correctly
4. No duplicate packages
5. No conflicting packages

## Options

| Option | Description |
|--------|-------------|
| `--verbose` | List all resolved packages |
| `--duplicates` | Check for duplicate packages |
| `--conflicts` | Check for cross-backend conflicts |

## Examples

### Basic Validation

```bash
declarch check
```

Output:
```
✓ Configuration file exists
✓ KDL syntax valid
✓ All imports resolved
✓ No duplicates found
```

### List All Packages

```bash
declarch check --verbose
```

Shows every package in your configuration.

### Find Duplicates

```bash
declarch check --duplicates
```

Warns if you declared the same package multiple times.

### Find Conflicts

```bash
declarch check --conflicts
```

Warns if the same package name exists in different backends (e.g., `ripgrep` in both AUR and cargo).

## Common Workflow

```bash
# After editing config
declarch edit
declarch check

# Before syncing
declarch check && declarch sync
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | Errors found |
| 2 | Warnings found |

Use in scripts:
```bash
declarch check && declarch sync
```

## Related

- [`edit`](edit.md) - Edit configuration
- [`sync`](sync.md) - Apply configuration
- [`info`](info.md) - View system status
