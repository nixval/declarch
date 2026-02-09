# Basic Concepts

## Three Core Ideas

### 1. Declare, Don't Command

Instead of running install commands, you declare what you want:

```kdl
pkg {
    aur {
        neovim
        firefox
    }
}
```

Not:
```bash
paru -S neovim firefox
```

### 2. Sync to Apply

After editing your config, run:

```bash
declarch sync
```

This makes your system match your config.

### 3. Modules Organize

Split your config into logical files:

```
modules/
├── base.kdl       # Essential tools
├── dev.kdl        # Development stuff
└── gaming.kdl     # Games
```

Import them in `declarch.kdl`:
```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

## Backends

Backends are package managers:

| Backend | Description | Example Packages |
|---------|-------------|------------------|
| `aur` | AUR helper (paru/yay) | `neovim`, `brave-bin` |
| `pacman` | Official repos | `firefox`, `git` |
| `flatpak` | Flatpak apps | `com.spotify.Client` |

Official backends (aur, pacman, flatpak) work out of the box.

## Key Commands

| Command | What It Does |
|---------|--------------|
| `declarch init` | Create initial config |
| `declarch install <pkg>` | Add package to config |
| `declarch sync` | Apply config to system |
| `declarch sync preview` | Preview changes |

## Config Files

| File | Purpose |
|------|---------|
| `declarch.kdl` | Main entry point, imports modules |
| `backends.kdl` | Backend definitions |
| `modules/*.kdl` | Your package lists |
| `state.json` | Tracks installed packages |

---

That's it! You now know the basics.
