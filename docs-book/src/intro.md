# Introduction

**Declarch** is a universal declarative package manager for Linux.

## What is Declarch?

Declarch lets you declare ALL your packages from different package managers in one file and keeps them in sync.

## Why Use Declarch?

**Without Declarch** (imperative, forgetful):
```bash
paru -S neovim
npm install -g typescript
flatpak install com.spotify.Client
cargo install ripgrep
# A month later: why did I install these?
```

**With Declarch** (declarative, clear):
```kdl
// One file, all your packages
packages {
  neovim          // From Arch/AUR
}

packages:npm {
  typescript      // From npm
}

packages:flatpak {
  com.spotify.Client
}

packages:cargo {
  ripgrep
}
```

Then just run:
```bash
declarch sync
```

## Three Ways to Write Packages

Declarch gives you **3 syntax options** - choose what you like:

### Option 1: Backend Blocks (Recommended)
```kdl
packages:npm {
  typescript
  prettier
}

packages:cargo {
  ripgrep
  fd-find
}
```

### Option 2: Embedded Blocks
```kdl
packages {
  npm {
    typescript
    prettier
  }

  cargo {
    ripgrep
    fd-find
  }
}
```

### Option 3: Inline Prefix
```kdl
packages {
  npm:typescript
  npm:prettier
  cargo:ripgrep
  cargo:fd-find
}
```

### Mix All Three Styles!
```kdl
packages {
  // Default (AUR)
  neovim
  git

  // Inline
  npm:prettier

  // Embedded block
  npm {
    typescript
    eslint
  }

  // Backend block
  flatpak {
    com.spotify.Client
  }
}
```

All three styles work the same. Pick what you prefer!

## What Happens When You Sync?

Declarch talks to each package manager:

```kdl
packages         → paru/yay (AUR)
packages:npm     → npm install -g
packages:cargo   → cargo install
packages:pip  → pip install
packages:flatpak → flatpak install
packages:soar    → soar install (cross-distro!)
```

It only installs what's missing and removes what you don't want anymore.

## Features

- **One config for all packages** - No more forgetting what you installed
- **Works on any Linux** - Use Soar backend for cross-distro packages
- **Safe** - See what will change before syncing
- **Flexible** - Create custom backends for any package manager
- **Modular** - Split config into multiple files
- **Smart** - Auto-adopts existing packages

## Who is This For?

- **Arch Linux users** who want declarative package management
- **Developers** managing tools across npm, cargo, pip, etc.
- **People with multiple machines** - Share configs between computers
- **Anyone** tired of forgetting why they installed packages

## Quick Example

Save this as `~/.config/declarch/declarch.kdl`:

```kdl
meta {
  host "my-laptop"
}

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

## Status

⚠️ **Declarch is in BETA**

- Only tested on Arch-based distros (Arch Linux, EndeavourOS)
- Expect breaking changes
- See [Troubleshooting](advanced/troubleshooting.html) for known issues

---

**Ready?** See [Installation](getting-started/installation.html).
