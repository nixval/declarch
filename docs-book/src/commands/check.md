# check

Verify configuration.

## Usage

```bash
declarch check [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `all` | Run all checks (default) |
| `duplicates` | Find duplicate packages |
| `conflicts` | Find cross-backend conflicts |
| `validate` | Check syntax only |

## Examples

### Check Everything

```bash
declarch check
```

Checks:
- Config syntax
- Imports exist
- Duplicate packages
- Cross-backend conflicts

### Check Duplicates

```bash
declarch check duplicates
```

Finds packages defined multiple times.

### Check Conflicts

```bash
declarch check conflicts
```

Finds packages with same name in different backends.

### Validate Syntax

```bash
declarch check validate
```

Just checks KDL syntax and imports.

## Fixing Errors

If check finds issues, it shows line numbers:

```
error: No closing '}' for child block
  --> ~/.config/declarch/base.kdl:8:5
   |
 8 │     meta {
   │      ^
```

Edit the file and fix the issue.
