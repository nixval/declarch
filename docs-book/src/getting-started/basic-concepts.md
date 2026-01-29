# Basic Concepts

Understanding the core concepts behind declarch's declarative package management.

## Imperative vs Declarative

### Imperative Package Management (Traditional)

You run commands to install packages:

```bash
# Day 1
paru -S bat exa ripgrep
npm install -g typescript prettier
flatpak install com.spotify.Client

# Month later...
# Why did I install this?
# Where did this come from?
# Do I still need it?
```

**Problems:**
- No record of why packages were installed
- Hard to reproduce setups on new machines
- Difficult to share configurations
- System becomes a "package graveyard"

### Declarative Package Management (Declarch)

You declare what you want in a config file:

```kdl
// ~/.config/declarch/declarch.kdl

packages {
    bat
    exa
    ripgrep
}

packages:npm {
    typescript
    prettier
}

packages:flatpak {
    com.spotify.Client
}
```

Then run one command:

```bash
declarch sync
```

**Benefits:**
- Clear record of all installed packages
- Easy to reproduce on new machines
- Simple to share with others
- System stays clean and organized

## How Declarch Works

### 1. Configuration (Desired State)

You define your desired state in `~/.config/declarch/declarch.kdl`:

```kdl
packages {
    vim
    bat
}
```

### 2. State Tracking (Actual State)

Declarch tracks installed packages in `~/.local/state/declarch/state.json`:

```json
{
  "packages": {
    "aur": ["vim", "bat"]
  }
}
```

### 3. Synchronization

When you run `declarch sync`:

| Config Has | System Has | Action |
|------------|------------|--------|
| ✅ vim | ❌ missing | **Install** vim |
| ✅ bat | ✅ installed | **Adopt** (track) |
| ❌ (removed) | ✅ old-pkg | **Keep** (unless `--prune`) |

## Key Behaviors

### Adoption, Not Reinstallation

If a package is already installed, declarch doesn't reinstall it:

```bash
# You already have bat installed manually
declarch sync

# Output:
✓ bat (aur) - already installed, adopted
```

This makes declarch safe to adopt existing systems.

### Safe Default: No Deletion

By default, declarch **only adds**, never removes:

```kdl
// Remove from config
packages {
    // bat removed
}
```

```bash
declarch sync
# bat stays installed! Not removed without --prune flag
```

To remove, use `--prune`:

```bash
declarch sync --prune
# Now bat will be removed
```

### Only Manages What It Knows

Declarch only removes packages it explicitly manages:

```bash
# You manually: paru -S random-tool
# Not in declarch config

declarch sync --prune
# random-tool stays! Declarch doesn't know about it
```

## The Configuration File

### Location

- Main config: `~/.config/declarch/declarch.kdl`
- Modules: `~/.config/declarch/modules/*.kdl`

### Format: KDL (Key Definition Language)

KDL is a human-readable configuration language:

```kdl
// Comments start with //

// Simple list
packages {
    bat
    exa
    ripgrep
}

// With backends
packages:flatpak {
    com.spotify.Client
}

// Advanced features
meta {
    description "My Workstation"
    author "nixval"
}

conflicts {
    vim neovim  // Can't have both
}

policy {
    protected {
        linux  // Never remove
    }
}
```

## Backends Architecture

Declarch supports multiple package managers through **backends**:

| Backend | Description | Syntax | Scope |
|---------|-------------|--------|-------|
| **aur** | Arch User Repository (default on Arch) | `packages { pkg }` | Arch-only |
| **soar** | Static binary registry | `packages:soar { pkg }` | All Linux |
| **flatpak** | Universal apps | `packages:flatpak { app }` | All Linux |
| **npm** | Node.js packages | `packages:npm { pkg }` | All Linux |
| **python** | Python packages | `packages:pip { pkg }` | All Linux |
| **cargo** | Rust crates | `packages:cargo { crate }` | All Linux |
| **brew** | Homebrew | `packages:brew { pkg }` | Linux/macOS |

### Default Backend

If you don't specify a backend, declarch uses the default for your system:

- **Arch-based**: AUR
- **Other**: Soar (if available)

## Module System

Break your config into reusable pieces:

**Main config** (`declarch.kdl`):
```kdl
imports {
    modules/base
    modules/desktop
    modules/development
}
```

**Module** (`modules/development.kdl`):
```kdl
meta {
    description "Development tools"
}

packages:npm {
    typescript
    prettier
}

packages:cargo {
    ripgrep
}
```

Benefits:
- Organize by category (desktop, development, gaming)
- Share across machines
- Mix and match configurations

## Remote Init

Fetch configs from Git repositories:

```bash
# GitHub
declarch init username/dotfiles

# GitLab
declarch init gitlab.com/username/dotfiles

# Specific variant
declarch init username/dotfiles:hyprland

# Official registry
declarch init hyprland/niri-nico
```

Similar to:
- **Go**: `import "github.com/user/repo"`
- **Nix**: Importing flake inputs

## Safety Features

### Dry Run Mode

Always preview before applying:

```bash
declarch sync --dry-run
```

### Protected Packages

Critical system packages are never removed:

```kdl
policy {
    protected {
        linux
        systemd
        base-devel
    }
}
```

### State Backups

Automatic backup before every state change:

```bash
~/.local/state/declarch/state.json.backup
```

### Hooks Security

Hooks (commands that run before/after sync) are **disabled by default**:

```bash
# Must explicitly enable
declarch sync --hooks
```

## Comparison: Traditional vs Declarch

| Task | Traditional | Declarch |
|------|-------------|----------|
| Install package | `paru -S bat` | Add to config, `declarch sync` |
| Remove package | `paru -R bat` | Remove from config, `declarch sync --prune` |
| Update system | `paru -Syu` | `declarch sync -u` |
| List installed | `paru -Qe` | `declarch info` |
| Share setup | Copy commands manually | Share config file |
| Reproduce setup | Run many commands | `declarch init <url>` |

## Summary

**Declarch = Git for Packages**

Just as Git tracks file changes, declarch tracks package changes:

| Git | Declarch |
|-----|----------|
| `git add` | Add to config |
| `git status` | `declarch check` |
| `git commit` | `declarch sync` |
| `git push` | Push config to GitHub |
| `git pull` | `declarch init <url>` |

Ready to dive deeper?
- [Commands Reference](../commands/)
- [Configuration Guide](../configuration/)
- [Advanced Topics](../advanced/)
