# Minimal Setup

The simplest way to get started with Declarch.

## Configuration

Save this as `~/.config/declarch/declarch.kdl`:

```kdl
// Basic metadata
meta {
  host "my-laptop"
}

// System packages (from Arch/AUR)
packages {
  neovim
  git
  curl
  htop
}
```

## That's It!

Run the commands:
```bash
# Initialize (creates config file)
declarch init

# Check what will be installed
declarch check

# Install everything
declarch sync
```

## Adding Other Package Managers

Declarch supports **many package managers**. Here's how to add them:

### npm (Node.js)
```kdl
// Style 1: Backend block (recommended)
packages:npm {
  typescript
  prettier
}

// Style 2: Embedded block
packages {
  npm {
    typescript
    prettier
  }
}

// Style 3: Inline
packages {
  npm:typescript
  npm:prettier
}
```

### cargo (Rust)
```kdl
packages:cargo {
  ripgrep
  fd-find
}
```

### Flatpak (apps)
```kdl
packages:flatpak {
  com.spotify.Client
}
```

### All Together

```kdl
meta {
  host "my-laptop"
}

// System packages
packages {
  neovim
  git
  curl
}

// Node.js tools
packages:npm {
  typescript
  prettier
}

// Rust tools
packages:cargo {
  ripgrep
  fd-find
}

// Desktop apps
packages:flatpak {
  com.spotify.Client
}
```

## What Each Backend Does

| Backend | Installs Via | Example |
|---------|--------------|---------|
| `packages` | paru/pacman (AUR) | `neovim` |
| `packages:npm` | `npm install -g` | `typescript` |
| `packages:cargo` | `cargo install` | `ripgrep` |
| `packages:flatpak` | `flatpak install` | `com.spotify.Client` |
| `packages:soar` | `soar install` | `bat` (any Linux!) |

## Next Steps

- Try [Desktop Environment](desktop.html) for a complete setup
- See [Development Environment](development.html) for programming tools
- Learn [Modular Setup](modular.html) to organize configs

## Source Files

- [`minimal.kdl`](https://github.com/nixval/declarch/blob/main/examples/minimal.kdl)
