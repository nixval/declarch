# Quick Start Guide

Get started with declarch in 5 minutes.

## Prerequisites

✅ Declarch installed ([Installation Guide](installation.md))
✅ Arch Linux or Arch-based distribution (for AUR backend)
✅ An AUR helper (paru or yay) installed

## Step 1: Initialize Configuration

Create your first declarch configuration:

```bash
declarch init
```

This creates:
- `~/.config/declarch/declarch.kdl` - Your main configuration file

## Step 2: Understand the Default Config

Let's look at what was created:

```bash
cat ~/.config/declarch/declarch.kdl
```

The default config includes example syntax (commented out) for reference.

## Step 3: Add Your First Packages

**NEW: Use the install command (recommended)**

```bash
# Add packages to modules/others.kdl
declarch install bat exa ripgrep fd

# Add to specific module
declarch install brave --module browsers

# Install from specific backend
declarch install soar:bat

# Multiple packages at once
declarch install bat fzf ripgrep --module base
```

The install command:
- Automatically adds packages to your KDL configuration
- Syncs only the specified module (efficient!)
- Shows backend information: `bat (aur)`
- Auto-rolls back on failure

**Or edit manually**:

Edit the configuration:

```bash
declarch edit
```

Or edit directly:
```bash
nano ~/.config/declarch/declarch.kdl
```

Add some packages:

```kdl
// ~/.config/declarch/declarch.kdl

// Default backend (AUR for Arch)
packages {
    bat
    exa
    ripgrep
    fd
}

// Flatpak applications
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

## Selective Module Sync

With the `--module` flag, you can install packages to specific modules without syncing your entire configuration:

```bash
# Install only to 'base' module
declarch install bat fzf --module base

# Install only to 'browsers' module
declarch install brave firefox --module browsers

# Install only to 'dev' module
declarch install nodejs python --module dev
```

This is more efficient - only the specified module is synced, not all modules.

## Automatic Rollback

If installation fails, declarch automatically restores your configuration:

```bash
$ declarch install soar:nonexistent --module others

Installing Packages
ℹ Syncing packages: nonexistent (soar) ...

Changes:
  Install: nonexistent (soar)
? Proceed? [Y/n] y
✗ nonexistent from soar not found
✗ 1 package(s) failed to install

# ✓ modules/others.kdl automatically restored
# ✓ No invalid entries left behind
# ✓ No manual cleanup needed
```

## Step 4: Preview Changes

Always preview before applying:

```bash
declarch sync --dry-run
```

Example output:
```
[DRY RUN] Would install:
  + bat (aur)
  + exa (aur)
  + ripgrep (aur)
  + fd (aur)
  + com.spotify.Client (flatpak)
  + org.mozilla.firefox (flatpak)

[DRY RUN] Would remove: (none)
```

## Step 5: Apply Changes

When satisfied, apply:

```bash
declarch sync
```

Declarch will:
1. Check which packages are already installed
2. Install missing packages
3. Adopt already-installed packages (track them without reinstalling)

## Step 6: Verify

Check what's being managed:

```bash
declarch info
```

Output:
```
=== Declarch System Info ===

Config: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Managed Packages: 6
  AUR: bat, exa, ripgrep, fd (4)
  Flatpak: com.spotify.Client, org.mozilla.firefox (2)
```

## Common Workflows

### Adding a New Package

1. Edit config:
   ```kdl
   packages {
       bat
       exa
       fzf  // ← Add this
   }
   ```

2. Preview and apply:
   ```bash
   declarch sync --dry-run
   declarch sync
   ```

### Removing a Package

1. Remove from config:
   ```kdl
   packages {
       bat
       exa
       // fzf removed
   }
   ```

2. To actually remove from system:
   ```bash
   declarch sync --prune
   ```

### Using Different Backends

```kdl
// AUR packages (default for Arch)
packages {
    hyprland
    waybar
}

// Cross-distro static binaries
packages:soar {
    bat
    exa
}

// Language ecosystem packages
packages:npm {
    typescript
    prettier
}

packages:pip {
    black
    ruff
}

packages:cargo {
    ripgrep
    fd-find
}

// Universal apps
packages:flatpak {
    com.spotify.Client
    com.discordapp.Discord
}
```

### System Update + Sync

Update system packages and sync config:

```bash
declarch sync -u
```

### Safe Cleanup

Remove packages that are in config but no longer wanted:

```bash
# Preview what would be removed
declarch sync --prune --dry-run

# Actually remove
declarch sync --prune
```

## Remote Init: Start from Existing Config

Instead of starting from scratch, fetch a pre-made config:

```bash
# From GitHub
declarch init myuser/hyprland-setup

# From official registry
declarch init hyprland/niri-nico

# Direct URL
declarch init https://example.com/my-config.kdl
```

Then customize it:

```bash
declarch edit
declarch sync --dry-run
declarch sync
```

## Next Steps

- [KDL Syntax Guide](../configuration/kdl-syntax.md) - Learn all configuration options
- [Commands Reference](../commands/) - Detailed command documentation
- [Advanced Topics](../advanced/) - Modules, hooks, custom backends

## Example: Complete Daily Workflow

```bash
# Morning coffee routine - update everything
declarch sync -u

# Discover new tool, add to config
declarch edit
# Add 'eza' to packages block

# Preview changes
declarch sync --dry-run

# Apply
declarch sync

# Later, decide to clean up
declarch sync --prune --dry-run
declarch sync --prune

# Check system status
declarch info
```
