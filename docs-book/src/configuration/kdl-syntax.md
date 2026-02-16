# KDL Basics

Declarch config is written in KDL.
This page only covers beginner syntax.

## Minimal example

```kdl
pkg {
    pacman {
        firefox
        git
    }
}
```

## Rules to remember

1. Blocks use `{}`.
2. Package names are plain entries inside backend blocks.
3. Quote string values in metadata/settings fields.

```kdl
meta {
    title "My Setup"
    description "My daily packages"
}
```

## Typical layout

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}

pkg {
    pacman { firefox }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

## Optional profile and host blocks

Use this only when you want extra packages for a specific situation.
If you do not use these blocks, nothing changes in default behavior.

```kdl
pkg {
    aur { git curl }
}

profile "desktop" {
    pkg {
        aur { hyprland waybar }
    }
}

host "vps-1" {
    pkg {
        aur { fail2ban tmux }
    }
}
```

Activate them explicitly from CLI:

```bash
declarch sync --profile desktop
declarch sync --host vps-1
declarch sync --profile desktop --host vps-1
```

Need full syntax details? Use [Syntax Reference (Advanced)](./syntax.md).
