# Introduction

**Declarch** is a universal declarative package manager for Linux.

## What is Declarch?

Declarch allows you to declare all your packages from different package managers (AUR, Flatpak, npm, cargo, pip, and custom backends) in one configuration file and keep them in sync.

## Why Declarative?

Traditional package management is **imperative**:

```bash
# Install packages one by one
paru -s neovim
npm install -g typescript
flatpak install com.spotify.Client
cargo install ripgrep
```

**Problem:** A month later, you forget why you installed them or where they came from.

With Declarch, it's **declarative**:

```kdl
// ~/.config/declarch/declarch.kdl
packages {
  neovim
  nodejs
  typescript  // backend: npm
  ripgrep     // backend: cargo
}

flatpak {
  com.spotify.Client
}
```

## Features

- **Universal**: Support AUR, Flatpak, npm, cargo, pip, and custom backends
- **Declarative**: One config file for all your packages
- **Cross-distro**: Works on Arch Linux and can be extended to other distros
- **Safe**: See what will be installed/removed before syncing
- **Flexible**: Create custom backends for any package manager

## Who is this for?

- **Arch Linux users** who want Nix-style declarative package management
- **Developers** who manage packages across multiple package managers
- **System administrators** who want reproducible system configurations
- **Linux enthusiasts** who want to experiment with declarative systems

## Philosophy

Declarch is inspired by [Nix](https://nixos.org/) and [Home Manager](https://nix-community.github.io/home-manager/), but designed to be:
- **Simpler**: No language to learn (KDL config is intuitive)
- **Non-intrusive**: Works with existing package managers
- **Flexible**: Easy to extend with custom backends

## Status

⚠️ **Declarch is in BETA**

- Architecture is still evolving
- Only tested on Arch-based distros (Arch Linux, EndeavourOS)
- Expect breaking changes
- See [Troubleshooting](advanced/troubleshooting.html) for known issues

---

**Ready to get started?** See [Installation](getting-started/installation.html).
