# Modular Configuration Setup

Split your configuration into multiple files for better organization.

## Directory Structure

```
~/.config/declarch/
├── declarch.kdl           # Main configuration
├── modules/
│   ├── base.kdl           # Essential packages
│   ├── desktop.kdl        # Desktop environment
│   ├── development.kdl    # Development tools
│   └── gaming.kdl         # Gaming-related
└── backends.kdl           # Custom backends
```

## Main Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
    host "my-workstation"
    description "Modular setup"
}

// Import modules
imports {
    "modules/base"
    "modules/desktop"
    "modules/development"
}

// Host-specific overrides
%host "gaming-pc" {
    imports {
        "modules/gaming"
    }
}
```

## Base Module

```kdl
// ~/.config/declarch/modules/base.kdl

// Essential system tools
packages {
    git
    curl
    wget
    vim
    htop
    tmux
    zsh
}
```

## Desktop Module

```kdl
// ~/.config/declarch/modules/desktop.kdl

packages {
    # Desktop environment
    hyprland
    waybar
    rofi-wayland
    alacritty
}

packages:flatpak {
    com.spotify.Client
    org.telegram.desktop
}
```

## Development Module

```kdl
// ~/.config/declarch/modules/development.kdl

packages:cargo {
    ripgrep
    fd-find
    bat
    zoxide
}

packages:npm {
    typescript
    prettier
}
```

## Gaming Module

```kdl
// ~/.config/declarch/modules/gaming.kdl

packages {
    steam
    lutris
    gamemode
    mangohud
}

// Host-specific: only on gaming-pc
%host "gaming-pc" {
    packages {
        wine-staging
        protonup-qt
    }
}
```

## Benefits of Modular Setup

### 1. Organization
- Separate concerns into different files
- Easy to find and update specific functionality
- Clear structure for complex configs

### 2. Reusability
- Share modules across different machines
- Mix and match as needed
- Create a "library" of modules

### 3. Host-Specific Configs
- Different packages for different machines
- Use `%host` directive for overrides
- Conditional imports per host

### 4. Version Control
- Easier to manage changes in git
- Clear history per module
- Smaller, focused pull requests

## Import Syntax

### Basic Import
```kdl
imports {
    "modules/base"
}
```

### Multiple Imports
```kdl
imports {
    "modules/base"
    "modules/desktop"
    "modules/development"
}
```

### Nested Paths
```kdl
imports {
    "modules/desktop/hyprland"
    "modules/development/rust"
    "modules/mydotfiles/alacritty"
}
```

## Host-Specific Overrides

### Conditional Block
```kdl
// Only apply on gaming-pc
%host "gaming-pc" {
    imports {
        "modules/gaming"
    }

    packages {
        steam
    }
}
```

### Multiple Hosts
```kdl
%host "desktop" "workstation" {
    packages {
        heavy-software
    }
}
```

## Editing Module Files

Use `declarch edit` to edit specific modules:

```bash
# Edit main config
declarch edit

# Edit specific module
declarch edit base
declarch edit desktop
declarch edit development/rust

# Edit with custom editor
declarch edit base --editor nvim
```

## Tips

1. **Start Simple**: Begin with single file, split later as needed
2. **Use Meaningful Names**: `base.kdl`, `desktop.kdl`, etc.
3. **Keep Imports Order**: Import base modules first
4. **Avoid Circular Imports**: Don't import files that import each other
5. **Document Modules**: Add meta info to each module file

## Example Module with Metadata

```kdl
// ~/.config/declarch/modules/development.kdl

meta {
    description "Development tools and IDEs"
    version "1.0.0"
    author "yourname"
    tags "development" "programming"
}

packages:cargo {
    ripgrep
    fd-find
}

on-sync "notify-send 'Dev tools updated'"
```

## Usage

```bash
# Check all imported configurations
declarch check

# Sync all modules
declarch sync

# Sync specific target (host)
declarch sync --target gaming-pc

# See what will be synced
declarch sync --dry-run
```

## Source Files

- [`modular.kdl`](https://github.com/nixval/declarch/blob/main/examples/modular.kdl)

---

**See also:**
- [KDL Syntax - Imports](../configuration/kdl-syntax.html#imports)
- [Modules](../configuration/modules.html)
- [Remote Init](../advanced/remote-init.html) - Import from remote URLs
