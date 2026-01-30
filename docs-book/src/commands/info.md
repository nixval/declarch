# declarch info

Show system status and managed packages.

## Usage

```bash
declarch info [OPTIONS]
```

## Options

- `--backend <BACKEND>` - Filter by backend (e.g., aur, flatpak, npm, cargo, pip)
- `--package <PACKAGE>` - Filter by package name
- `--format <FORMAT>` - Output format (table, json, yaml)
- `--doctor` - Diagnose system issues
- `--debug` - Enable verbose logging

## Quick Start

```bash
# Show all packages
declarch info

# Filter by backend
declarch info --backend aur

# Filter by package name
declarch info --package bat

# Output as JSON
declarch info --format json

# Output as YAML
declarch info --format yaml
```

## Filtering Examples

### Show Only AUR Packages

```bash
declarch info --backend aur
```

### Show Only npm Packages

```bash
declarch info --backend npm
```

### Search for Specific Package

```bash
declarch info --package bat
```

### Combine Filters

```bash
# Show npm packages containing "typescript"
declarch info --backend npm --package typescript
```

## Output Formats

### Table Format (default)

```bash
declarch info
```

```
System Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Hostname: mymachine
Last Sync: 2026-01-30 14:30:00

Total Managed 15
• AUR/Repo:  5
• Flatpak:   3
• Soar:      2
• NPM:       3
• Cargo:     2

Managed Packages:
  → bat
  → exa
  flt → flatpak-1
  ...
```

### JSON Format

```bash
declarch info --format json
```

```json
{
  "meta": {
    "hostname": "mymachine",
    "last_sync": "2026-01-30T14:30:00Z"
  },
  "packages": {
    "aur:bat": {
      "backend": "Aur",
      "config_name": "bat",
      "version": "0.24.0"
    }
  }
}
```

### YAML Format

```bash
declarch info --format yaml
```

Useful for configuration management and parsing.

## Doctor Mode

```bash
declarch info --doctor
```

Diagnoses:
- Config file existence and validity
- State file existence and consistency
- Orphan packages
- Backend availability (paru, yay, flatpak, cargo, npm, bun, pip)
- State consistency checks

## Common Uses

### Check Status Before/After Sync

```bash
declarch info
declarch sync
declarch info
```

### Export Package List

```bash
# Export as JSON
declarch info --format json > packages.json

# Export as YAML
declarch info --format yaml > packages.yaml
```

### Audit Specific Backend

```bash
# Check all AUR packages
declarch info --backend aur

# Check all npm packages
declarch info --backend npm
```

### Search Packages

```bash
# Find all packages containing "git"
declarch info --package git
```

## Related

- [`check`](check.md) - Validate configuration
- [`sync`](sync.md) - Install packages
- [`list`](list.md) - List installed packages
