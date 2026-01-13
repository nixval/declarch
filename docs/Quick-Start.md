# Quick Start Guide

Get started with declarch in 5 minutes.

---

## Prerequisites

✅ Declarch installed ([see Installation Guide](Installation.md))
✅ Arch Linux or Arch-based distribution
✅ AUR helper (paru or yay)

---

## 5-Minute Setup

### Step 1: Initialize Configuration

```bash
declarch init
```

This creates `~/.config/declarch/declarch.kdl`

---

### Step 2: Edit Configuration

```bash
declarch edit
```

Add your packages:

```kdl
// CLI tools
packages {
    bat
    exa
    fd
    ripgrep
}

// AUR packages
packages:aur {
    hyprland
    waybar
}

// Flatpak apps
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

---

### Step 3: Verify Syntax

```bash
declarch check
```

Output:
```
✓ Configuration is valid
✓ Found 3 Soar packages
✓ Found 2 AUR packages
✓ Found 2 Flatpak packages
```

---

### Step 4: Preview Changes

```bash
declarch sync --dry-run
```

This shows what will be installed without actually installing.

---

### Step 5: Apply

```bash
declarch sync
```

Declarch will now install all packages.

---

## Basic Commands

### View Status

```bash
declarch info
```

Shows:
- Installed packages
- Package counts by backend
- Configuration location

---

### Check Configuration

```bash
declarch check
```

Validates syntax and shows package counts.

---

### Edit Config

```bash
declarch edit
```

Opens `declarch.kdl` in your editor.

---

### Synchronize

```bash
declarch sync
```

Installs/removes packages to match your config.

**Useful flags:**
```bash
declarch sync --dry-run     # Preview only
declarch sync -u            # Update system first (paru -Syu)
declarch sync --prune       # Remove unlisted packages
declarch sync --gc          # Garbage collect orphans after sync
```

---

## Common Workflows

### Add New Package

1. Edit config:
   ```bash
   declarch edit
   ```

2. Add package:
   ```kdl
   packages {
       bat
       exa
       my-new-package  # Add here
   }
   ```

3. Sync:
   ```bash
   declarch sync
   ```

---

### Remove Package

1. Edit config:
   ```bash
   declarch edit
   ```

2. Remove package from config

3. Sync:
   ```bash
   declarch sync
   ```

4. (Optional) Remove orphan dependencies:
   ```bash
   declarch sync --gc
   ```

---

### Switch Package Variant

Replace a package with an alternative:

```bash
declarch switch firefox firefox-beta
```

Or manually:
1. Edit config: remove `firefox`, add `firefox-beta`
2. Run: `declarch sync`

---

### Update System

```bash
# Update system packages, then sync
declarch sync -u
```

This runs `paru -Syu` before installing packages.

---

## Remote Init (Fetch from GitHub)

Fetch a pre-made configuration:

```bash
# From GitHub user repository
declarch init jakoolit/dotfiles

# From official project
declarch init hyprwm/hyprland

# From community registry
declarch init hyprland/niri-nico

# From GitLab
declarch init gitlab.com/user/repo

# With specific variant
declarch init jakoolit/dotfiles:uwsm

# From specific branch
declarch init jakoolit/dotfiles/develop:uwsm
```

See [Remote Init Guide](Remote-Init-Guide.md) for details.

---

## Organizing Configuration

### Single File

Simple setup, all packages in one file:

**`~/.config/declarch/declarch.kdl`**
```kdl
packages:aur {
    hyprland
}

packages {
    bat
}
```

---

### Modular Setup

Organize into multiple files:

**`~/.config/declarch/declarch.kdl`**
```kdl
packages {
    bat
    exa
}

imports {
    modules/hyprland
    modules/gaming
    modules/development
}
```

**`~/.config/declarch/modules/hyprland.kdl`**
```kdl
packages:aur {
    hyprland
    waybar
}
```

**`~/.config/declarch/modules/gaming.kdl`**
```kdl
packages:flatpak {
    com.valvesoftware.Steam
}
```

---

### Host-Specific Configs

Different packages for different machines:

**`~/.config/declarch/declarch.kdl`**
```kdl
// Base packages for all hosts
packages {
    bat
    exa
    ripgrep
}

// Host-specific
imports {
    hosts/desktop  # Only on desktop
    hosts/laptop   # Only on laptop
}
```

See [Examples](Examples.md) for more.

---

## Understanding Backends

Declarch supports multiple package backends:

### Soar (Cross-Distro Static Binaries)

```kdl
packages {
    bat
    exa
    fd
}
```

- ✅ Works on any Linux
- ✅ No installation dependencies
- ✅ Fast, reliable

**Learn more:** [Soar Integration](https://github.com/nixval/declarch/blob/main/SOAR_INTEGRATION_SUMMARY.md)

---

### AUR (Arch User Repository)

```kdl
packages:aur {
    hyprland
    waybar
}
```

- ✅ Huge package selection
- ⚠️ Arch Linux only
- ⚠️ Requires AUR helper (paru/yay)

---

### Flatpak (Universal Packages)

```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

- ✅ Works on any Linux
- ✅ Sandboxed applications
- ⚠️ Requires Flatpak

---

## Configuration Syntax

### Declaring Packages

**Multiple ways:**

```kdl
// Method 1: Direct arguments
packages bat exa ripgrep

// Method 2: Block format
packages {
    bat
    exa
    ripgrep
}

// Method 3: With backend
packages:aur {
    hyprland
    waybar
}

// Method 4: Embedded (mixed)
packages {
    bat          // Soar
    aur {
        hyprland  // AUR
    }
    flatpak {
        com.spotify.Client  // Flatpak
    }
}
```

See [KDL Syntax Reference](KDL-Syntax-Reference.md) for complete syntax.

---

### Setting Editor

```kdl
editor "nvim"
```

Used by `declarch edit` command.

**Priority:**
1. `editor "nvim"` in declarch.kdl
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default)

---

### Package Aliases

Map config name to actual package:

```kdl
aliases-pkg {
    pipewire pipewire-jack2
}

packages:aur {
    pipewire  # Will install pipewire-jack2
}
```

---

### Excluding Packages

```kdl
excludes {
    unwanted-package
}
```

Prevents specific packages from being installed, even in imported modules.

---

## Next Steps

Now that you know the basics:

1. **Explore [KDL Syntax Reference](KDL-Syntax-Reference.md)** - Complete syntax documentation
2. **Browse [Examples](Examples.md)** - Real-world configurations
3. **Set up [Remote Init](Remote-Init-Guide.md)** - Share configs via GitHub
4. **Read [Repository Requirements](Repository-Requirements.md)** - Make your own repo compatible

---

## Troubleshooting

### Configuration Error

```bash
declarch check
```

Shows syntax errors and their location.

---

### Package Not Found

1. Check package name spelling
2. Verify package exists in the specified backend
3. For AUR: Make sure AUR helper is installed
4. For Flatpak: Make sure Flathub is added

---

### Editor Not Working

Set editor in config:
```kdl
editor "nvim"
```

Or set environment variable:
```bash
export EDITOR=nvim
```

---

## Help and Community

- **Documentation**: [Home](Home.md)
- **GitHub Issues**: https://github.com/nixval/declarch/issues
- **Config Registry**: https://github.com/nixval/declarch-packages

---

**Ready to declarch your system? 🚀**
