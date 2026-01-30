# declarch info

Show system status and managed packages.

## Usage

```bash
declarch info
```

## Quick Start

```bash
declarch info
```

## What It Shows

```
=== Declarch System Info ===

Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Configured Packages: 15
  AUR: 5 packages
  Flatpak: 3 packages
  npm: 4 packages
  Cargo: 3 packages

Managed Packages: 12
  AUR: 4 packages
  Flatpak: 3 packages
  npm: 3 packages
  Cargo: 2 packages

Unadopted Packages: 3
  AUR: hyprland, waybar
  npm: typescript
```

## Understanding the Output

**Configured Packages** - Packages in your configuration files

**Managed Packages** - Packages being tracked by declarch (synced at least once)

**Unadopted Packages** - Packages in config but not yet synced (will be installed on next `declarch sync`)

## Common Uses

### Check Status Before/After Sync

```bash
declarch info | grep "Unadopted"
declarch sync
declarch info | grep "Unadopted"
```

### Verify All Packages Managed

```bash
declarch info
# If "Unadopted Packages: 0", all configured packages are installed
```

### Quick Overview

```bash
declarch info
```

Shows complete system status at a glance.

## Related

- [`check`](check.md) - Validate configuration
- [`sync`](sync.md) - Install unadopted packages
