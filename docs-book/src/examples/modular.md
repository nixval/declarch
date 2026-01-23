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

packages {
  # Essential system tools
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

flatpak {
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

%host "gaming-pc" {
  packages {
    wine-staging
    protonup-qt
  }
}
```

## Benefits of Modular Setup

### 1. **Organization**
- Separate concerns into different files
- Easy to find and update specific functionality

### 2. **Reusability**
- Share modules across different machines
- Mix and match as needed

### 3. **Host-Specific Configs**
- Different packages for different machines
- Use `%host` directive for overrides

### 4. **Version Control**
- Easier to manage changes in git
- Clear history per module

## Usage

```bash
# Check all imported configurations
declarch check

# Sync all modules
declarch sync

# Sync specific target (host)
declarch sync --target gaming-pc
```

## Tips

1. **Start Simple**: Begin with single file, split later as needed
2. **Use Meaningful Names**: `base.kdl`, `desktop.kdl`, etc.
3. **Keep Imports Order**: Import base modules first
4. **Avoid Circular Imports**: Don't import files that import each other

## Source Files

- [`modular.kdl`](https://github.com/nixval/declarch/blob/main/examples/modular.kdl)

---

**See also:**
- [KDL Syntax - Imports](../configuration/kdl-syntax.html#imports)
- [Modules](../configuration/modules.html)
