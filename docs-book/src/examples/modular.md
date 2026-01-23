# Modular Configuration Setup

Split your configuration into multiple files for better organization.

## Quick Example

Instead of one giant file, split it into modules:

```kdl
// Main file: ~/.config/declarch/declarch.kdl
meta {
  host "my-workstation"
}

// Import other files
imports {
  "modules/base"
  "modules/desktop"
  "modules/development"
}
```

## Module Files

Create these files in `~/.config/declarch/modules/`:

**Base system** (`modules/base.kdl`):
```kdl
packages {
  git
  curl
  vim
  htop
}
```

**Desktop** (`modules/desktop.kdl`):
```kdl
packages {
  hyprland
  waybar
}

packages:flatpak {
  com.spotify.Client
}
```

**Development** (`modules/development.kdl`):
```kdl
packages:cargo {
  ripgrep
  fd-find
}

packages:npm {
  typescript
  prettier
}
```

## Why Use Modules?

### 1. Organization
- Keep related packages together
- Easier to find things
- Less overwhelming

### 2. Reusability
- Share modules between computers
- Mix and match as needed
- Build a "module library"

### 3. Maintenance
- Update one module at a time
- Clear git history
- Easier to troubleshoot

## Usage

```bash
# Edit main config
declarch edit

# Edit specific module
declarch edit base
declarch edit desktop
declarch edit development

# Sync everything
declarch sync
```

## Host-Specific Modules

Only use certain modules on specific machines:

```kdl
// Main config
imports {
  "modules/base"
  "modules/desktop"
  "modules/development"
}

// Only on gaming PC
%host "gaming-pc" {
  imports {
    "modules/gaming"
  }

  packages {
    steam
    lutris
  }
}
```

## Three Syntax Styles

You can use any of the three syntax styles in modules:

**Style 1: Backend blocks**:
```kdl
// modules/development.kdl
packages:cargo {
  ripgrep
}

packages:npm {
  typescript
}
```

**Style 2: Embedded blocks**:
```kdl
// modules/development.kdl
packages {
  cargo {
    ripgrep
  }

  npm {
    typescript
  }
}
```

**Style 3: Inline**:
```kdl
// modules/development.kdl
packages {
  cargo:ripgrep
  npm:typescript
}
```

## Tips

1. Start simple, split later
2. Use meaningful names: `base.kdl`, `desktop.kdl`
3. Import base modules first
4. Don't create circular imports

## Directory Structure

```
~/.config/declarch/
├── declarch.kdl      # Main config
└── modules/
    ├── base.kdl       # Essential packages
    ├── desktop.kdl    # Desktop environment
    ├── development.kdl # Dev tools
    └── gaming.kdl     # Games (optional)
```

## Source Files

- [`modular.kdl`](https://github.com/nixval/declarch/blob/main/examples/modular.kdl)

---

**See also:**
- [KDL Syntax](../configuration/kdl-syntax.html) - Complete syntax reference
- [Modules](../configuration/modules.html) - Advanced module features
