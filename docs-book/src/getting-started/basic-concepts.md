# Basic Concepts

Understanding how declarch works.

## The Main Idea

**Traditional way (imperative):**
```bash
paru -S bat exa ripgrep
# Oops, why did I install these again?
```

**Declarch way (declarative):**
```bash
# Add to config file
declarch install bat exa ripgrep

# Your system is now in sync with config
```

## How It Works

### 1. Declare What You Want

Edit `~/.config/declarch/declarch.kdl`:

```kdl
packages {
    bat
    exa
    ripgrep
}
```

Or use the install command:
```bash
declarch install bat exa ripgrep
```

### 2. Sync Your System

```bash
declarch sync
```

Declarch will:
- ✅ Install missing packages
- ✅ Track already-installed packages
- ✅ Remove packages you deleted from config (with `--prune`)

## One File, Many Backends

Single config, multiple package managers:

```kdl
// AUR packages
packages {
    neovim
}

// Node.js packages
packages:npm {
    typescript
}

// Python packages
packages:pip {
    black
}

// Flatpak apps
packages:flatpak {
    com.spotify.Client
}
```

All managed with one command: `declarch sync`

## Modules: Keep It Organized

Split packages into logical groups:

```kdl
// declarch.kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/gaming.kdl"
}
```

```kdl
// modules/base.kdl
packages {
    bat
    exa
    ripgrep
}
```

```kdl
// modules/dev.kdl
packages:npm {
    nodejs
    typescript
}
```

Benefits:
- 📁 Organized by purpose
- 🔄 Easy to enable/disable modules
- 📦 Share specific setups with others

## Why Use Declarch?

**Reproducibility:**
```bash
# Get a new machine
git clone my-dotfiles.git
# Copy declarch config to ~/.config/declarch/
declarch sync
# Done! All packages installed
```

**Clarity:**
```bash
# See what you have
declarch info

# See what's in a module
declarch info --module base
```

**Safety:**
- Config file is your documentation
- Easy to review and edit
- No mysterious packages

## Next Steps

- [Quick Start](quick-start.md) - Get started now
- [Modules Guide](../configuration/modules.md) - Organize your config
