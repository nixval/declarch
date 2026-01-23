# Modules

Organize your declarch configuration into reusable, modular components.

## What Are Modules?

Modules are separate `.kdl` files that you can import into your main configuration. They help you:

- Organize packages by category (desktop, development, gaming)
- Share configurations across machines
- Mix and match different setups
- Keep your config clean and maintainable

## Module Structure

### Directory Layout

```
~/.config/declarch/
├── declarch.kdl          # Main config
└── modules/
    ├── base.kdl
    ├── desktop.kdl
    ├── development.kdl
    │   ├── rust.kdl
    │   ├── python.kdl
    │   └── node.kdl
    ├── gaming.kdl
    └── work.kdl
```

## Basic Module Usage

### Creating a Module

Create `~/.config/declarch/modules/base.kdl`:

```kdl
// modules/base.kdl

meta {
    description "Base system packages"
    author "nixval"
}

packages {
    bat
    exa
    ripgrep
    fd
    fzf
}

packages:soar {
    zoxide
    dust
}
```

### Importing a Module

In your main `declarch.kdl`:

```kdl
// declarch.kdl

imports {
    modules/base
}

// Now all base.kdl packages are included
```

### Multiple Imports

```kdl
imports {
    modules/base
    modules/desktop
    modules/development
    modules/gaming
}
```

## Module Organization

### By Category

```
modules/
├── base.kdl          # Essential CLI tools
├── desktop.kdl       # Desktop environment
├── development.kdl   # Development tools
├── gaming.kdl        # Gaming-related
└── media.kdl         # Audio/video tools
```

### By Backend

```
modules/
├── aur.kdl
├── flatpak.kdl
├── npm.kdl
├── python.kdl
└── rust.kdl
```

### By Purpose/Role

```
modules/
├── personal.kdl      # Personal machine
├── work.kdl          # Work machine
├── server.kdl        # Server setup
└── laptop.kdl        # Laptop-specific
```

### Nested (Recommended for Large Setups)

```
modules/
├── base.kdl
├── desktop/
│   ├── hyprland.kdl
│   └── kde.kdl
├── development/
│   ├── rust.kdl
│   ├── python.kdl
│   ├── node.kdl
│   └── go.kdl
└── gaming/
    ├── steam.kdl
    └── lutris.kdl
```

## Module Features

### Module Metadata

```kdl
// modules/development/rust.kdl

meta {
    description "Rust development tools"
    author "nixval"
    version "1.0.0"
    tags "development" "rust"
}
```

### Module Packages

```kdl
// modules/development/rust.kdl

packages:cargo {
    ripgrep
    fd-find
    cargo-edit
    cargo-watch
    sqlx-cli
}

packages {
    rust-analyzer
}
```

### Module-Specific Configuration

```kdl
// modules/desktop/hyprland.kdl

meta {
    description "Hyprland desktop environment"
}

options:aur {
    noconfirm
}

packages {
    hyprland
    waybar
    wofi
    hyprpaper
    hyprlock
}

packages:flatpak {
    com.spotify.Client
}

on-sync "notify-send 'Hyprland packages updated'"
```

## Advanced Module Features

### Module Merging Behavior

When you import multiple modules, their configurations are merged:

| Feature | Merging Behavior |
|---------|-----------------|
| **Meta** | First config wins (usually root) |
| **Packages** | All packages accumulated |
| **Conflicts** | Accumulated from all configs |
| **Backend Options** | Later configs override earlier |
| **Environment** | Later configs extend earlier |
| **Repositories** | Later configs extend earlier |
| **Policy** | Last one wins |
| **Hooks** | Later configs extend earlier |

### Example Merging

**base.kdl:**
```kdl
meta {
    description "Base packages"
}

packages {
    bat
    exa
}

env EDITOR="nvim"
```

**desktop.kdl:**
```kdl
meta {
    description "Desktop packages"
}

packages {
    hyprland
    waybar
}

env VISUAL="nvim"
```

**Result after importing both:**
```kdl
// Meta: "Base packages" (first wins)
// Packages: bat, exa, hyprland, waybar (accumulated)
// Environment: EDITOR="nvim", VISUAL="nvim" (extended)
```

### Module Inheritance

Modules can import other modules:

```kdl
// modules/development.kdl

imports {
    modules/development/rust
    modules/development/python
    modules/development/node
}
```

## Real-World Examples

### Minimal Setup

```
~/.config/declarch/
├── declarch.kdl
└── modules/
    └── base.kdl
```

**declarch.kdl:**
```kdl
imports {
    modules/base
}
```

**modules/base.kdl:**
```kdl
packages {
    bat
    exa
    ripgrep
}
```

### Desktop Workstation

```
modules/
├── base.kdl
├── desktop/
│   └── hyprland.kdl
└── development/
    ├── rust.kdl
    ├── node.kdl
    └── python.kdl
```

**declarch.kdl:**
```kdl
imports {
    modules/base
    modules/desktop/hyprland
    modules/development/rust
    modules/development/node
}
```

### Multi-Machine Setup

**Desktop (declarch.kdl):**
```kdl
imports {
    modules/base
    modules/desktop/hyprland
    modules/development
    modules/gaming
}
```

**Laptop (declarch.kdl):**
```kdl
imports {
    modules/base
    modules/desktop/kde
    modules/development
    modules/laptop  # Battery optimization, etc.
}
```

**Server (server.kdl):**
```kdl
imports {
    modules/base
    modules/server
}
```

## Editing Modules

### Edit Main Config

```bash
declarch edit
```

Opens: `~/.config/declarch/declarch.kdl`

### Edit Module

```bash
declarch edit base
```

Opens: `~/.config/declarch/modules/base.kdl`

### Edit Nested Module

```bash
declarch edit development/rust
```

Opens: `~/.config/declarch/modules/development/rust.kdl`

## Module Best Practices

### 1. Keep Modules Focused

```kdl
// ✅ Good: Single purpose
// modules/rust.kdl - Only Rust-related packages

// ❌ Bad: Too broad
// modules/everything.kdl - All packages in one file
```

### 2. Use Descriptive Names

```kdl
// ✅ Good
modules/development/rust.kdl
modules/desktop/hyprland.kdl

// ❌ Bad
modules/stuff.kdl
modules/packages.kdl
```

### 3. Document Dependencies

```kdl
// modules/development/kde.kdl

// Requires: modules/desktop/kde
// This module adds KDE-specific development tools

packages {
    kdevelop
    kate
}
```

### 4. Use Conflicts for Mutually Exclusive Setups

```kdl
// modules/desktop/hyprland.kdl
conflicts {
    // Can't use both hyprland and kde packages
    kde-plasma
}
```

### 5. Share Common Base

```kdl
// All machines import base
imports {
    modules/base
}

// Then add machine-specific modules
imports {
    modules/desktop/hyprland  // Desktop only
}
```

## Module Templates

### Base Module Template

```kdl
// modules/base.kdl

meta {
    description "Essential CLI tools"
    author "yourname"
    tags "base" "cli"
}

packages {
    bat
    exa
    ripgrep
    fd
    fzf
    zoxide
    dust
}

packages:soar {
    bat
    exa
}
```

### Desktop Module Template

```kdl
// modules/desktop/hyprland.kdl

meta {
    description "Hyprland desktop environment"
    author "yourname"
    tags "desktop" "wayland" "hyprland"
}

packages {
    hyprland
    waybar
    wofi
    waypaper
    wlogout
}

packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

### Development Module Template

```kdl
// modules/development/rust.kdl

meta {
    description "Rust development environment"
    author "yourname"
    tags "development" "rust"
}

packages:cargo {
    ripgrep
    fd-find
    cargo-edit
    cargo-watch
}

packages {
    rust-analyzer
}
```

## Troubleshooting

### Module Not Found

```bash
$ declarch check
✗ Failed to resolve import: modules/missing.kdl
```

**Solution:** Create the module or remove from imports.

### Circular Imports

```bash
$ declarch check
✗ Circular import detected: modules/a → modules/b → modules/a
```

**Solution:** Remove circular dependency. Refactor common packages into a separate module.

### Conflicting Packages

```bash
$ declarch check --duplicates
⚠ Found duplicate: vim in modules/base.kdl:5, modules/desktop.kdl:10
```

**Solution:** Remove from one module or create a shared module.

## Module Management Workflow

### Add New Module

```bash
# 1. Create module file
touch ~/.config/declarch/modules/new-module.kdl

# 2. Edit module
declarch edit new-module

# 3. Import in main config
declarch edit
# Add: imports { modules/new-module }

# 4. Check and sync
declarch check
declarch sync --dry-run
declarch sync
```

### Remove Module

```bash
# 1. Remove import from main config
declarch edit
# Remove: modules/new-module

# 2. Optionally delete module file
rm ~/.config/declarch/modules/new-module.kdl

# 3. Sync to remove its packages (if desired)
declarch sync --prune
```

### Split Large Module

```bash
# Start with one large module
modules/development.kdl  # All dev tools

# Split into smaller ones
modules/development/rust.kdl
modules/development/python.kdl
modules/development/node.kdl

# Create parent that imports children
modules/development.kdl:
imports {
    modules/development/rust
    modules/development/python
    modules/development/node
}
```

## Related

- [KDL Syntax Reference](kdl-syntax.md) - Configuration syntax
- [Backends Reference](backends.md) - Supported package managers
- [Remote Init](../advanced/remote-init.md) - Share modules via Git

## See Also

- [Module Examples](../examples/modular.kdl) - Complete modular configuration
