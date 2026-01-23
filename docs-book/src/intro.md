# Introduction

**Declarch** is a universal declarative package manager for Linux.

## What is Declarch?

Declarch allows you to declare all your packages from different package managers (AUR, Flatpak, npm, cargo, pip, and custom backends) in one configuration file and keep them in sync.

## Why Declarative?

Traditional package management is **imperative**:

```bash
# Install packages one by one, forget why later
paru -S neovim
npm install -g typescript
flatpak install com.spotify.Client
cargo install ripgrep
```

**Problem:** A month later, you forget why you installed them or where they came from.

With Declarch, it's **declarative**:

```kdl
// ~/.config/declarch/declarch.kdl

// Default backend (AUR on Arch Linux)
packages {
    neovim
    git
    curl
}

// Node.js packages
packages:npm {
    typescript
    prettier
}

// Rust crates
packages:cargo {
    ripgrep
    fd-find
}

// Flatpak applications
packages:flatpak {
    com.spotify.Client
}
```

Then simply run:
```bash
declarch sync
```

## How It Works

Declarch uses **backend syntax** to specify which package manager to use:

```kdl
packages              // Default backend (AUR on Arch)
packages:npm          // Node.js packages
packages:cargo        // Rust crates
packages:python       // Python packages (pip)
packages:flatpak      // Flatpak applications
packages:soar         // Static binaries (cross-distro)
```

Each backend installs packages through its respective package manager:
- **AUR** → `paru`/`yay`
- **npm** → `npm install -g`
- **cargo** → `cargo install`
- **python** → `pip install`
- **flatpak** → `flatpak install`
- **soar** → `soar install`

## Features

- **Universal**: Support AUR, Flatpak, npm, cargo, pip, soar, and custom backends
- **Declarative**: One config file for all your packages
- **Cross-distro**: Soar backend works on any Linux distro
- **Safe**: See what will be installed/removed before syncing
- **Flexible**: Create custom backends for any package manager
- **Modular**: Import and organize configs into reusable modules
- **Smart**: Auto-adopts existing packages, only installs what's missing

## Who is this for?

- **Arch Linux users** who want Nix-style declarative package management
- **Developers** who manage packages across multiple package managers
- **System administrators** who want reproducible system configurations
- **Linux enthusiasts** who want to experiment with declarative systems

## Philosophy

Declarch is inspired by [Nix](https://nixos.org/) and [Home Manager](https://nix-community.github.io/home-manager/), but designed to be:
- **Simpler**: KDL config is intuitive and readable
- **Non-intrusive**: Works with existing package managers
- **Flexible**: Easy to extend with custom backends
- **Pragmatic**: Doesn't require reinstalling your entire system

## Status

⚠️ **Declarch is in BETA**

- Architecture is evolving
- Only tested on Arch-based distros (Arch Linux, EndeavourOS)
- Expect breaking changes
- See [Troubleshooting](advanced/troubleshooting.html) for known issues

---

**Ready to get started?** See [Installation](getting-started/installation.html).
