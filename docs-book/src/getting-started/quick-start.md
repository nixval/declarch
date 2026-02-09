# Quick Start

Get up and running in 3 minutes.

## Step 1: Initialize

```bash
declarch init
```

Output:
```
Initializing declarch root
Created config directory: ~/.config/declarch
Created backends configuration: ~/.config/declarch/backends.kdl
Created config file: ~/.config/declarch/declarch.kdl
Created modules directory: ~/.config/declarch/modules
Created base module: ~/.config/declarch/modules/base.kdl
Initialized state for host: myhostname
```

This creates:
```
~/.config/declarch/
├── declarch.kdl          # Main config - imports modules
├── backends.kdl          # Backend definitions (aur, pacman, flatpak)
├── modules/
│   └── base.kdl          # Your packages go here
└── state.json            # Tracks installed packages
```

## Step 2: Add Packages

### Method A: Command Line

```bash
# Add single package
declarch install neovim

# Add multiple packages
declarch install bat fzf ripgrep fd

# Add with backend prefix
declarch install aur:neovim
```

### Method B: Edit Config Directly

Open `~/.config/declarch/modules/base.kdl`:

```kdl
pkg {
    aur {
        neovim
        bat
        fzf
        ripgrep
        fd
    }
}
```

## Step 3: Sync

```bash
declarch sync
```

You'll see:
```
Changes:
  Adopt: (aur): neovim, bat, fzf, ripgrep, fd

? Proceed with sync? [Y/n]
```

Type `y` and Enter. Declarch will install all packages.

## What Just Happened?

1. **Declarch read your config** and saw you want those packages
2. **It checked what's already installed** on your system
3. **It showed you the changes** it would make
4. **You confirmed**, and it installed the packages
5. **It recorded the state** so it knows what's managed

## Next Steps

- Learn about [backends](../configuration/backends.md)
- Organize with [modules](../configuration/modules.md)
- See all [commands](../commands/index.md)
