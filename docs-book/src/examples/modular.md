# Modular Setup

Organize by purpose, not by backend.

## Philosophy

Group packages by **what you use them for**, not where they come from.

## Structure

```
modules/
├── core.kdl          # Always loaded
├── work.kdl          # Work-specific
├── gaming.kdl        # Games only
└── media.kdl         # Photos/video
```

## Example: core.kdl

```kdl
pkg {
    aur {
        neovim
        bat
        fzf
    }
    
    pacman {
        git
        curl
    }
    
    flatpak {
        org.mozilla.firefox
    }
}
```

## Example: work.kdl

```kdl
pkg {
    pacman {
        slack-desktop
        zoom
    }
    
    flatpak {
        com.microsoft.Teams
    }
}
```

## Example: gaming.kdl

```kdl
pkg {
    aur {
        steam
        lutris
        gamemode
    }
    
    flatpak {
        com.valvesoftware.Steam
    }
}
```

## Loading Selectively

```kdl
// declarch.kdl - Work laptop
imports {
    "modules/core.kdl"
    "modules/work.kdl"
    // "modules/gaming.kdl"  # Disabled
}
```

```kdl
// declarch.kdl - Gaming PC
imports {
    "modules/core.kdl"
    "modules/gaming.kdl"
    "modules/media.kdl"
}
```

## Benefits

- **Per-machine configs** - Load only what you need
- **Git-friendly** - Track different setups in branches
- **Shareable** - Share work module with colleagues
