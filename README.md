# declarch

**Declarative Package Manager for Arch Linux**

[![CI/CD](https://github.com/nixval/declarch/actions/workflows/ci.yml/badge.svg)](https://github.com/nixval/declarch/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

declarch is a declarative package management system for Arch Linux that allows you to define your installed packages in KDL configuration files and sync your system to match.

## Features

- **Declarative Configuration**: Define packages in KDL files instead of running ad-hoc commands
- **Multi-Backend Support**: AUR, Soar, Flatpak, npm, yarn, pnpm, bun, pip, cargo, Homebrew
- **Automatic Dependency Resolution**: Handles cross-backend dependencies
- **Conflict Detection**: Prevents conflicting packages from being installed
- **Hooks System**: Run custom scripts before/after sync operations
- **Modular Configuration**: Organize packages into logical modules (base, gaming, work, etc.)
- **Selective Sync**: Install packages to specific modules without syncing everything
- **Automatic Rollback**: Failed installations restore your config files automatically

## Quick Start

```bash
# Install declarch
paru -S declarch

# Initialize with a community module
declarch init shell/dms

# Install packages (automatically adds to modules/others.kdl)
declarch install bat fzf ripgrep

# Install to specific module
declarch install brave --module browsers

# Sync your system
declarch sync

# Update system and sync packages
declarch sync --update
```

## Selective Module Sync

Install packages to specific modules and sync only that module:

```bash
# Install to 'base' module only
declarch install bat --module base

# Install to 'others' module only
declarch install vim --module others

# Multiple packages to same module
declarch install bat fzf ripgrep --module base
```

This is more efficient - only the specified module is synced, not your entire configuration.

## Automatic Rollback

If installation fails, declarch automatically restores KDL files:

```bash
# Attempted installation fails
declarch install soar:nonexistent --module others

# ✓ KDL file automatically restored
# ✓ No manual cleanup needed
# ✓ Clear error message shown
```

## Configuration

### Module Structure

```
~/.config/declarch/
├── declarch.kdl          # Root configuration
├── state.json            # Installed package state
└── modules/              # Package modules
    ├── base.kdl          # Base system packages
    ├── gaming.kdl        # Gaming packages
    └── work.kdl          # Work-related packages
```

### Example KDL Module

```kdl
// modules/base.kdl
packages {
  bat
  fzf
  ripgrep
  fd
  exa
}

// Environment variables
env "EDITOR" {
  nvim
}

// Hooks
hooks pre-sync {
  echo "Starting sync..."
}
```

## Commands

### install

Add packages to your configuration:

```bash
# Basic install (adds to modules/others.kdl)
declarch install <package>

# Install specific backend
declarch install soar:bat

# Install to specific module
declarch install bat --module base

# Multiple packages
declarch install bat fzf ripgrep

# Install without syncing
declarch install bat --no-sync
```

### sync

Synchronize your system with the configuration:

```bash
# Sync all modules
declarch sync

# Sync with system update
declarch sync --update

# Sync specific module
declarch sync --modules base

# Dry run (preview changes)
declarch sync --dry-run

# Force package removal
declarch sync --prune
```

### init

Initialize a new configuration:

```bash
# Initialize from community module
declarch init shell/dms

# Initialize empty config
declarch init

# Force overwrite existing
declarch init --force
```

### info

Show package information:

```bash
# Show all packages
declarch info

# Show specific module
declarch info --module base

# Show backend info
declarch info --backend aur
```

### list

List available modules and packages:

```bash
# List all modules
declarch list modules

# List packages in module
declarch list packages --module base

# List all packages
declarch list packages
```

### check

Validate configuration:

```bash
# Check for conflicts
declarch check

# Check specific module
declarch check --module base

# Verbose output
declarch check --verbose
```

### hooks

Manage hooks:

```bash
# List all hooks
declarch hooks list

# Run hooks manually
declarch hooks run

# Test hooks
declarch hooks test
```

### switch

Switch between configurations:

```bash
# Switch host profile
declarch switch endeavour

# Dry run
declarch switch --dry-run laptop
```

### settings

Manage configuration:

```bash
# Show all settings
declarch settings show

# Get setting
declarch settings get color

# Set setting
declarch settings set color always

# Reset setting
declarch settings reset color
```

## Output Examples

### Before (Verbose)

```bash
$ declarch install bat

Installing Packages
⚠ Skipping missing import: /path/to/base.kdl
✓ Created: modules/others.kdl
ℹ Packages: bat
ℹ Syncing system...

Synchronizing Packages
ℹ Scanning system state...
────────────────────────────────────────────────────────────
Changes:
  Adopt:   bat (aur)
? Proceed? [Y/n] ✓ Sync complete!
```

### After (Concise)

```bash
$ declarch install bat

Installing Packages
ℹ Syncing packages: bat (aur) ...

Changes:
  Adopt:   bat (aur)
? Proceed? [Y/n] ✓ Sync completed, added to 'others.kdl'
```

## Installation

### From AUR

```bash
paru -S declarch
```

### From Source

```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

## Documentation

- [Full Documentation](docs-book/)
- [Hooks Guide](docs-book/guide/hooks.md)
- [Module System](docs-book/guide/modules.md)
- [Configuration Reference](docs-book/reference/config.md)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
