# declarch check

Validate your configuration.

## Usage

```bash
declarch check [OPTIONS]
```

## Options

- `--verbose` / `-v` - List all resolved packages (global flag)
- `--duplicates` - Check for duplicate package declarations
- `--conflicts` - Check for cross-backend package name conflicts
- `--only-duplicates` - Only check for duplicate declarations
- `--only-conflicts` - Only check for cross-backend conflicts
- `--backend <BACKEND>` - Validate specific backend only
- `--diff` - Show planned changes without executing
- `--validate` - Validate config only, don't sync
- `--benchmark` - Show performance metrics

## Quick Start

```bash
# Basic validation
declarch check

# Check for duplicates
declarch check --duplicates

# Check for conflicts
declarch check --conflicts

# Check duplicates only (skip conflicts)
declarch check --only-duplicates

# Check conflicts only (skip duplicates)
declarch check --only-conflicts
```

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

### Selective Validation

```bash
# Only check for duplicates (faster)
declarch check --only-duplicates

# Only check for conflicts (faster)
declarch check --only-conflicts

# Check both (default when flags are used)
declarch check --duplicates --conflicts
```

Note: `--only-duplicates` and `--only-conflicts` are mutually exclusive.

### Filter by Backend

```bash
# Check only AUR packages
declarch check --backend aur

# Check only npm packages
declarch check --backend npm
```

### Show Planned Changes

```bash
declarch check --diff
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
declarch check --duplicates --conflicts --verbose
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
declarch check --duplicates || echo "Duplicates found!"
```

## Related

- [`edit`](edit.md) - Edit configuration
- [`sync`](sync.md) - Apply configuration
- [`info`](info.md) - View system status
- [`validate`](https://github.com/nixval/declarch) - Alternative to check
