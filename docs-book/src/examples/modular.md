# Modular Example

Organize by purpose, not by backend.

## Example structure

```text
modules/
├── core.kdl
├── work.kdl
├── gaming.kdl
└── media.kdl
```

## `core.kdl`

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

## `work.kdl`

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

## Select imports per machine

```kdl
// work laptop
imports {
    "modules/core.kdl"
    "modules/work.kdl"
}
```

```kdl
// gaming pc
imports {
    "modules/core.kdl"
    "modules/gaming.kdl"
    "modules/media.kdl"
}
```

## Why this pattern works

- cleaner diffs
- easier reuse
- machine-specific setups stay simple
