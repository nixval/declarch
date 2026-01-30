# Introduction

**Declarch** is a declarative package manager for Linux.

## What is Declarch?

Declarch lets you declare all your packages in one file and keeps them in sync.

## Why Use Declarch?

**Without Declarch:**
```bash
paru -S neovim
npm install -g typescript
flatpak install com.spotify.Client
# A month later: why did I install these?
```

**With Declarch:**
```kdl
// One file, all your packages
packages {
  neovim
}

packages:npm {
  typescript
}

packages:flatpak {
  com.spotify.Client
}
```

Then just run:
```bash
declarch sync
```

## Quick Example

Save this as `~/.config/declarch/declarch.kdl`:

```kdl
packages {
  bat     // Modern cat
  exa     // Modern ls
  ripgrep // Fast search
}

packages:npm {
  typescript
  prettier
}

packages:flatpak {
  com.spotify.Client
}
```

Then run:
```bash
declarch sync
```

That's it! All your packages are now managed declaratively.

## Features

- **One config for all packages** - AUR, npm, flatpak, cargo, pip, and more
- **Safe** - See what will change before syncing
- **Modular** - Split config into multiple files
- **Smart** - Auto-adopts existing packages

## Who is This For?

- Arch Linux users who want declarative package management
- Developers managing tools across multiple package managers
- People with multiple machines who want to share configs
- Anyone tired of forgetting why they installed packages

## Status

⚠️ **Declarch is in BETA**

- Tested on Arch-based distros (Arch Linux, EndeavourOS)
- See [Troubleshooting](advanced/troubleshooting.html) for known issues

---

**Ready?** See [Installation](getting-started/installation.html).
