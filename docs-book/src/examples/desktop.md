# Desktop Environment

A complete desktop setup with GUI apps.

## Structure

```
modules/
├── base.kdl       # CLI tools
├── desktop.kdl    # Window manager
└── apps.kdl       # GUI applications
```

## modules/base.kdl

```kdl
pkg {
    aur {
        neovim
        bat
        fzf
        ripgrep
        fd
        zoxide
    }
    
    pacman {
        git
        curl
        wget
    }
}
```

## modules/desktop.kdl

```kdl
pkg {
    aur {
        hyprland
        waybar
        rofi-wayland
        swww
    }
    
    pacman {
        foot
        mako
        grim
        slurp
        wl-clipboard
    }
}
```

## modules/apps.kdl

```kdl
pkg {
    pacman {
        firefox
        thunderbird
    }
    
    flatpak {
        com.spotify.Client
        com.discordapp.Discord
        org.telegram.desktop
    }
}
```

## declarch.kdl

```kdl
imports {
    "modules/base.kdl"
    "modules/desktop.kdl"
    "modules/apps.kdl"
}
```

## Usage

```bash
declarch sync
```

Installs everything: CLI tools, Hyprland desktop, and GUI apps.
