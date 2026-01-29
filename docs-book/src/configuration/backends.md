# Package Backends

Complete reference for all supported package manager backends in declarch.

## Overview

Declarch supports multiple package management backends through a unified interface. Each backend represents a different package manager or ecosystem.

## Architecture

Declarch uses **two different backend implementation patterns** to support different types of package managers:

### Custom Rust Implementations

Some backends require custom Rust implementations due to their complexity or special requirements:

| Backend | Why Custom? | Location |
|---------|-------------|----------|
| **AUR** | AUR helper detection (paru/yay), special AUR handling | `src/packages/aur.rs` |
| **Flatpak** | Remote management, special installation patterns | `src/packages/flatpak.rs` |
| **Soar** | Auto-installation, static binary management | `src/packages/soar.rs` |

These backends have:
- Complex state management
- Special detection/initialization logic
- Non-standard command patterns
- Backend-specific features

### Generic Config-Driven Implementations

Most backends use a generic configuration-driven approach via `GenericManager`:

| Backend | Type | Config Location |
|---------|------|-----------------|
| **npm, yarn, pnpm, bun** | Node.js package managers | `src/backends/registry.rs` |
| **pip** | Python package manager | `src/backends/registry.rs` |
| **cargo** | Rust package manager | `src/backends/registry.rs` |
| **brew** | Homebrew | `src/backends/registry.rs` |

These backends:
- Follow standard package manager patterns
- Use simple install/remove/list commands
- Require no special initialization
- Can be fully configured declaratively

### How Backends Are Loaded

1. **Custom backends** are registered in `src/packages/registry.rs`
2. **Generic backends** load their configuration from `src/backends/registry.rs`
3. Both are created through the `BackendRegistry` factory
4. User-defined backends (from `backends.kdl`) can override built-in configurations

### Adding a New Backend

**For a custom backend** (complex logic):
1. Create implementation file in `src/packages/<name>.rs`
2. Implement `PackageManager` trait
3. Add `Backend::<Name>` to `core/types.rs`
4. Register in `BackendRegistry::register_defaults()`

**For a generic backend** (simple commands):
1. Add configuration to `src/backends/registry.rs::get_builtin_backends()`
2. Add `Backend::<Name>` to `core/types.rs`
3. Register in `BackendRegistry::register_defaults()` using `GenericManager`

## Available Backends

| Backend | Description | Distribution Support | Syntax |
|---------|-------------|---------------------|--------|
| **aur** | Arch User Repository | Arch-based | `packages { pkg }` |
| **soar** | Static binary registry | All Linux | `packages:soar { pkg }` |
| **flatpak** | Universal application packages | All Linux | `packages:flatpak { app }` |
| **npm** | Node.js packages | All Linux | `packages:npm { pkg }` |
| **yarn** | Yarn packages | All Linux | `packages:yarn { pkg }` |
| **pnpm** | pnpm packages | All Linux | `packages:pnpm { pkg }` |
| **bun** | Bun packages | All Linux | `packages:bun { pkg }` |
| **pip** | Python packages (pip) | All Linux | `packages:pip { pkg }` |
| **cargo** | Rust crates | All Linux | `packages:cargo { crate }` |
| **brew** | Homebrew | Linux, macOS | `packages:brew { pkg }` |

## AUR (Arch User Repository)

### Description

Default backend for Arch-based distributions. Installs packages from the Arch User Repository using an AUR helper.

### Requirements

- Arch Linux or Arch-based distribution
- AUR helper: `paru` or `yay`

### Installation

```bash
# Install AUR helper
paru -S paru  # or: paru -S yay
```

### Configuration

```kdl
// Default on Arch - no backend prefix needed
packages {
    hyprland
    waybar
    wofi
}

// Explicit
packages:aur {
    hyprland
    waybar
}
```

### Backend Options

```kdl
options:aur {
    noconfirm        // Skip confirmation prompts
    helper "paru"    // Use paru instead of yay
}
```

### Environment Variables

```kdl
env:aur MAKEFLAGS="-j4"  // Parallel builds
```

## Soar (Static Binaries)

### Description

Cross-distribution static binary registry. Works on any Linux distribution.

### Requirements

- Soar CLI installed
- Internet connection

### Installation

```bash
# Install Soar CLI
paru -S soar
# Or from GitHub releases
```

### Configuration

```kdl
packages:soar {
    bat
    exa
    ripgrep
    fd
    fzf
}
```

### Why Use Soar?

- Cross-distro compatibility
- Static binaries (no dependencies)
- Fast installation
- Reproducible builds

## Flatpak

### Description

Universal application packaging system for Linux. Works on any distribution with Flatpak support.

### Requirements

- Flatpak installed
- Flathub repository added

### Installation

```bash
# Install Flatpak
sudo pacman -S flatpak

# Add Flathub
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

### Configuration

```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    com.discordapp.Discord
    org.telegram.desktop
}
```

### Backend Options

```kdl
options:flatpak {
    noconfirm
}
```

### Package Naming

Flatpak packages use reverse domain notation:

```
com.spotify.Client
org.mozilla.firefox
com.discordapp.Discord
org.gimp.GIMP
```

Find packages at: [flathub.org](https://flathub.org)

## npm (Node.js)

### Description

Node.js package manager. Installs global Node.js packages.

### Requirements

- Node.js and npm installed

### Installation

```bash
paru -S nodejs npm
```

### Configuration

```kdl
packages:npm {
    typescript
    prettier
    eslint
    @types/node
}
```

### Global Install Location

Packages are installed to:
- `~/.npm-global/bin/` (if configured)
- Or system location

Setup:
```bash
npm config set prefix '~/.npm-global'
export PATH="~/.npm-global/bin:$PATH"
```

## Yarn

### Description

Alternative Node.js package manager by Facebook.

### Requirements

- Yarn installed

### Installation

```bash
npm install -g yarn
# or
paru -S yarn
```

### Configuration

```kdl
packages:yarn {
    eslint
    prettier
}
```

## pnpm

### Description

Fast, disk space efficient package manager for Node.js.

### Requirements

- pnpm installed

### Installation

```bash
npm install -g pnpm
# or
paru -S pnpm
```

### Configuration

```kdl
packages:pnpm {
    typescript
    prettier
}
```

## Bun

### Description

Fast all-in-one JavaScript runtime, package manager, bundler, and test runner.

### Requirements

- Bun installed

### Installation

```bash
curl -fsSL https://bun.sh/install | bash
```

### Configuration

```kdl
packages:bun {
    @types/node
    prettier
}
```

## Python (pip)

### Description

Python package installer.

### Requirements

- Python and pip installed

### Installation

```bash
paru -S python python-pip
```

### Configuration

```kdl
packages:pip {
    black
    ruff
    jupyter
    numpy
    pandas
}
```

### Virtual Environments

For project-specific packages, use virtual environments instead of global pip.

## Cargo (Rust)

### Description

Rust package manager for crates.

### Requirements

- Rust and Cargo installed

### Installation

```bash
paru -S rust cargo
# or via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Configuration

```kdl
packages:cargo {
    ripgrep
    fd-find
    cargo-edit
    cargo-watch
}
```

## Brew (Homebrew)

### Description

Cross-platform package manager. Works on Linux and macOS.

### Requirements

- Homebrew installed

### Installation

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Configuration

```kdl
packages:brew {
    vim
    ffmpeg
    imagemagick
}
```

## Cross-Distro Backends

The following backends work on **any** Linux distribution:

- ✅ **Soar** - Static binaries
- ✅ **Flatpak** - Universal apps
- ✅ **npm** - Node.js packages
- ✅ **yarn** - Yarn packages
- ✅ **pnpm** - pnpm packages
- ✅ **bun** - Bun packages
- ✅ **python** - Python packages
- ✅ **cargo** - Rust crates
- ✅ **brew** - Homebrew

The **AUR** backend only works on Arch-based distributions.

## Default Backend

If you don't specify a backend, declarch uses:

- **Arch-based systems**: AUR
- **Other systems**: Soar (if available)

Example:
```kdl
// On Arch: installs via AUR
packages {
    bat
}

// On Ubuntu: installs via Soar
packages {
    bat
}
```

## Custom Backends

You can define your own backends for any package manager:

```kdl
// Custom backend for nala (Debian/Ubuntu)
backends {
    nala {
        cmd "sudo nala install"
        list_cmd "nala list --installed"
        remove_cmd "sudo nala remove"
    }
}

// Use custom backend
packages:nala {
    vim
    ffmpeg
}
```

See [Custom Backends Guide](../advanced/custom-backends.md) for details.

## Backend Availability

Declarch automatically detects which backends are available:

```bash
$ declarch check

✓ AUR backend available (paru)
✓ Flatpak backend available
✓ npm backend available
✗ Python backend not available (python not found)
✓ Cargo backend available
```

If a backend isn't available, packages for that backend are skipped during sync.

## Mixing Backends

You can use multiple backends in the same configuration:

```kdl
// System packages (Arch)
packages {
    hyprland
    waybar
}

// Cross-distro CLI tools
packages:soar {
    bat
    exa
    ripgrep
}

// Universal apps
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}

// Development tools
packages:npm {
    typescript
    prettier
}

packages:pip {
    black
    ruff
}

packages:cargo {
    fd-find
}
```

## Package Name Conflicts

The same package name can exist in multiple backends:

```kdl
packages {
    ripgrep  // AUR: /usr/bin/ripgrep
}

packages:cargo {
    ripgrep  // Cargo: ~/.cargo/bin/ripgrep
}

packages:soar {
    ripgrep  // Soar: ~/.local/bin/ripgrep
}
```

Check for conflicts:
```bash
declarch check --conflicts
```

Your PATH ordering determines which version runs!

## Performance

### Package Count Checks

| Backend | Check Speed (100 packages) |
|---------|---------------------------|
| AUR | ~2-3 seconds |
| Soar | ~1 second |
| Flatpak | ~3-4 seconds |
| npm | ~1-2 seconds |
| Python | ~1-2 seconds |
| Cargo | ~2-3 seconds |

### Optimization Tips

1. **Use targeted syncs:**
   ```bash
   declarch sync --target flatpak
   ```

2. **Group packages by backend:**
   ```kdl
   // ✅ Efficient
   packages:npm { typescript prettier eslint }

   // ❌ Less efficient (mixed)
   packages { npm:typescript aur:hyprland }
   ```

## Related

- [KDL Syntax Reference](kdl-syntax.md) - Configuration syntax
- [Custom Backends](../advanced/custom-backends.md) - Define your own backends
- [Modules Guide](modules.md) - Organize configs by backend

## See Also

- [AUR Website](https://aur.archlinux.org/)
- [Soar Registry](https://soar.dev/)
- [Flathub](https://flathub.org/)
- [npm Registry](https://www.npmjs.com/)
- [PyPI](https://pypi.org/)
- [ crates.io](https://crates.io/)
