# declarch list

List installed packages.

## Usage

```bash
declarch list [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `all` - List all packages (default)
- `orphans` - List orphan packages
- `synced` - List synced packages

## Options

- `-b, --backend <BACKEND>` - Filter by backend (e.g., aur, flatpak, cargo, npm)
- `--format <FORMAT>` - Output format (table, json, yaml)

## Quick Start

```bash
# List all packages
declarch list

# List orphan packages (installed but not in config)
declarch list orphans

# List synced packages (in config and installed)
declarch list synced

# Filter by backend
declarch list --backend aur
```

## What It Lists

1. **All packages** - All managed packages
2. **Orphans** - Packages installed but not defined in config
3. **Synced** - Packages that match your configuration

## Examples

### List All Packages

```bash
declarch list
```

Lists all installed packages managed by declarch.

### Find Orphan Packages

```bash
declarch list orphans
```

Shows packages that are installed on your system but not defined in your declarch configuration. These are typically packages you installed manually outside of declarch.

### Check Synced Packages

```bash
declarch list synced
```

Shows packages that are both defined in your configuration and currently installed on the system.

### Filter by Backend

```bash
# List AUR packages only
declarch list --backend aur

# List Flatpak apps only
declarch list --backend flatpak

# List npm packages only
declarch list --backend npm
```

### Find Orphans in Specific Backend

```bash
# Find orphan AUR packages
declarch list orphans --backend aur

# Find orphan npm packages
declarch list orphans --backend npm
```

### Output as JSON

```bash
declarch list --format json
```

Useful for scripting and parsing.

### Output as YAML

```bash
declarch list --format yaml
```

Useful for configuration management.

## Common Workflows

### Clean Up Orphans

```bash
# First, see what orphans exist
declarch list orphans

# Review and decide which to remove
# Then use your package manager directly to remove them
paru - <package_name>
```

### Verify Sync Status

```bash
# Check what's synced
declarch list synced

# Compare with configuration
declarch check
```

### Audit Specific Backend

```bash
# Check all npm packages
declarch list --backend npm

# Check npm orphans
declarch list orphans --backend npm
```

## Migration from Flags

Old flag-based syntax is deprecated but still works:

```bash
# Old (deprecated, will be removed in v0.7.0)
declarch list --orphans
declarch list --synced

# New (recommended)
declarch list orphans
declarch list synced
```

## Related

- [`info`](info.md) - View detailed package information
- [`check`](check.md) - Validate configuration
- [`sync`](sync.md) - Synchronize packages
- [`install`](install.md) - Add packages to configuration
