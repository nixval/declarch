# Minimal Setup

Smallest useful setup for beginners.

## `declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
}
```

## `modules/base.kdl`

```kdl
pkg {
    aur {
        neovim
        git
    }
}
```

## Apply

```bash
declarch --dry-run sync
declarch sync
```

## Why this is a good start

- only one module
- easy to read
- easy to expand later


# Desktop Example

Simple desktop-oriented module layout.

## Suggested structure

```text
modules/
├── base.kdl
├── desktop.kdl
└── apps.kdl
```

## `modules/base.kdl`

```kdl
pkg {
    aur {
        neovim
        bat
        fzf
        ripgrep
    }

    pacman {
        git
        curl
        wget
    }
}
```

## `modules/desktop.kdl`

```kdl
pkg {
    aur {
        hyprland
        waybar
        rofi-wayland
    }

    pacman {
        foot
        mako
        grim
        slurp
    }
}
```

## `modules/apps.kdl`

```kdl
pkg {
    pacman {
        firefox
        thunderbird
    }

    flatpak {
        com.spotify.Client
        org.telegram.desktop
    }
}
```

## `declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/desktop.kdl"
    "modules/apps.kdl"
}
```

## Apply safely

```bash
declarch --dry-run sync
declarch sync
```


# Development Example

Beginner-friendly dev setup with separate modules.

## Prerequisite

```bash
declarch init --backend npm
```

## Structure

```text
modules/
├── base.kdl
├── dev.kdl
└── langs.kdl
```

## `modules/dev.kdl`

```kdl
pkg {
    aur {
        neovim
        tmux
        docker
    }

    pacman {
        git
        github-cli
        jq
    }
}
```

## `modules/langs.kdl`

```kdl
pkg {
    aur {
        rustup
    }

    npm {
        typescript
        ts-node
        prettier
        eslint
    }
}
```

## `declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/langs.kdl"
}
```

## Apply

```bash
declarch --dry-run sync
declarch sync
```


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


