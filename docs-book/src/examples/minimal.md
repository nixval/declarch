# Minimal Setup

This example shows the simplest possible Declarch configuration.

## Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
  host "my-laptop"
}

// Default packages from system repository
packages {
  neovim
  git
  curl
  wget
  htop
}
```

## What This Does

- Sets hostname to `my-laptop`
- Installs packages using system package manager (pacman on Arch)
- Tracks these packages in declarative config

## Usage

```bash
# Initialize config
declarch init

# Check what will be installed
declarch check

# Sync packages
declarch sync
```

## Source Files

- [`minimal.kdl`](https://github.com/nixval/declarch/blob/main/examples/minimal.kdl)

---

**Next:** See [Desktop Environment](desktop.html) for a more complete setup.
