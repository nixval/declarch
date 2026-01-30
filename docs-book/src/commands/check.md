# declarch check

Validate your configuration.

## Usage

```bash
declarch check [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `all` - Run all checks (default)
- `duplicates` - Check for duplicate package declarations
- `conflicts` - Check for cross-backend package name conflicts
- `validate` - Validate syntax only

## Options

- `--verbose` / `-v` - List all resolved packages (global flag)
- `--backend <BACKEND>` - Validate specific backend only
- `--diff` - Show planned changes without executing
- `--benchmark` - Show performance metrics
- `--modules <MODULES>` - Load additional modules temporarily

## Quick Start

```bash
# Run all checks (default)
declarch check

# Run all checks (explicit)
declarch check all

# Check for duplicates only
declarch check duplicates

# Check for conflicts only
declarch check conflicts

# Validate syntax only
declarch check validate
```

## What It Checks

1. **Configuration File** - Exists and is readable
2. **KDL Syntax** - Valid syntax and structure
3. **Imports** - All module imports resolve correctly
4. **Duplicates** - No duplicate package declarations (with `all` or `duplicates`)
5. **Conflicts** - No cross-backend package name conflicts (with `all` or `conflicts`)

## Examples

### Basic Validation

```bash
declarch check
```

Output:
```
Configuration Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Entry point: /home/user/.config/declarch/declarch.kdl
✓ Syntax & Imports: OK
Unique Packages: 10
Excludes: 0
```

### Find Duplicates

```bash
declarch check duplicates
```

Warns if you declared the same package multiple times:

```
Checking for duplicates...
⚠ Found 2 duplicate package declarations:
  📦 aur:ripgrep
     └─ base.kdl
     └─ development.kdl
```

### Find Conflicts

```bash
declarch check conflicts
```

Warns if the same package name exists in different backends:

```
Checking for cross-backend conflicts...
⚠ Found 1 package name conflicts across backends:

These packages have the same name but different backends:
They will be installed separately by each backend.
Watch out for PATH conflicts!

  ⚠ ripgrep
     └─ Aur

## What It Checks

1. **Configuration File** - Exists and is readable
2. **KDL Syntax** - Valid syntax and structure
3. **Imports** - All module imports resolve correctly
4. **Duplicates** - No duplicate package declarations
5. **Conflicts** - No cross-backend package name conflicts

## Examples

### Basic Validation

```bash
declarch check
```

Output:
```
Configuration Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Entry point: /home/user/.config/declarch/declarch.kdl
✓ Syntax & Imports: OK
✓ Configuration is valid
```

### Find Duplicates

```bash
declarch check --duplicates
```

Warns if you declared the same package multiple times:

```
Checking for duplicates...
⚠ Found 2 duplicate package declarations:
  📦 aur:ripgrep
     └─ base.kdl
     └─ development.kdl
```

### Find Conflicts

```bash
declarch check --conflicts
```

Warns if the same package name exists in different backends:

```
Checking for cross-backend conflicts...
⚠ Found 1 package name conflicts across backends:

These packages have the same name but different backends:
They will be installed separately by each backend.
Watch out for PATH conflicts!

  ⚠ ripgrep
     └─ Aur
     └─ Cargo

Example:
  If 'ripgrep' exists in both AUR and npm:
    • AUR installs to: /usr/bin/ripgrep
    • npm installs to:  ~/.npm-global/bin/ripgrep
  The one that runs depends on your PATH
```

### Validate Syntax Only

```bash
declarch check validate
```

Validates configuration file syntax and imports without checking for duplicates or conflicts.

### Filter by Backend

```bash
# Check only AUR packages
declarch check duplicates --backend aur

# Check only npm packages
declarch check conflicts --backend npm
```

### Show Planned Changes

```bash
declarch check all --diff
```

Shows what would change on sync without running it.

## Common Workflow

```bash
# After editing config
declarch edit
declarch check

# Before syncing
declarch check && declarch sync

# Full validation with verbose output
declarch check all -v
```

## Migration from Flags

Old flag-based syntax is deprecated but still works:

```bash
# Old (deprecated, will be removed in v0.7.0)
declarch check --duplicates
declarch check --conflicts
declarch check --validate

# New (recommended)
declarch check duplicates
declarch check conflicts
declarch check validate
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | Errors found |

Use in scripts:
```bash
# Only sync if config is valid
declarch check && declarch sync

# Check for specific issues
declarch check duplicates || echo "Duplicates found!"
```

## Related

- [`edit`](edit.md) - Edit configuration
- [`sync`](sync.md) - Apply configuration
- [`info`](info.md) - View system status

