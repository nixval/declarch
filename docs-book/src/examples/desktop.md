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
declarch sync preview
declarch sync
```
