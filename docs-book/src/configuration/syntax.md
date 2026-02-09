# Configuration Syntax

Complete reference for declarch KDL configuration files.

## File Structure

```
~/.config/declarch/
├── declarch.kdl       # Main config with imports
├── backends.kdl       # Backend definitions
├── modules/
│   └── base.kdl       # Package modules
└── state.json         # Tracked packages (auto-generated)
```

## Basic Syntax

### Comments
```kdl
// Single line comment

/*
  Multi-line comment
*/
```

### Strings
All string values must be quoted:
```kdl
// Correct
title "My Config"
format "whitespace"

// Wrong - will error
title My Config
format whitespace
```

## Main Config (declarch.kdl)

### Meta Block
```kdl
meta {
    title "My Setup"
    description "Development workstation"
    author "your-name"
    version "1.0.0"
}
```

### Imports
Import other config files:
```kdl
imports {
    "backends.kdl"
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

### Package Declaration
The `pkg` block contains all your packages:

```kdl
pkg {
    // Backend-specific blocks
    aur {
        neovim
        bat
        fzf
    }
    
    pacman {
        firefox
        git
    }
    
    flatpak {
        com.spotify.Client
    }
}
```

### Inline Backend Syntax
For single packages, use colon syntax:
```kdl
pkg:aur {
    neovim
}

// Same as:
pkg {
    aur {
        neovim
    }
}
```

## Backend Config (backends.kdl)

### Official Backends (Built-in)
```kdl
backend "aur" {
    meta {
        title "AUR Helper"
        description "Arch User Repository"
    }
    
    binary "paru" "yay"
    
    list "{binary} -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "{binary} -S --needed {packages}"
    remove "{binary} -R {packages}"
    
    fallback "pacman"
}
```

### Custom Backend Imports
```kdl
imports {
    "backends/npm.kdl"
    "backends/cargo.kdl"
}
```

## Advanced Features

### Conflicts
Mark packages that conflict with each other:
```kdl
conflicts {
    vim neovim
    pipewire pulseaudio
}
```

### Excludes
Packages to exclude from sync:
```kdl
excludes {
    unwanted-package
    another-package
}
```

### Environment Variables
Set env vars for package operations:
```kdl
env "EDITOR" {
    nvim
}

// Or with backend prefix
env:paru {
    MAKEFLAGS "-j4"
}
```

### Backend Options
Configure backend-specific options:
```kdl
options:aur {
    noconfirm
}
```

### Repositories
Add custom repositories:
```kdl
repos:aur {
    "https://aur.archlinux.org"
}
```

### Hooks
Run commands at specific times:
```kdl
on-sync "notify-send 'Sync complete'"
on-pre-sync "backup-configs.sh"
```

### Policy
Configure sync behavior:
```kdl
policy {
    protected {
        linux
        systemd
    }
    orphans "ask"  // or "remove", "keep"
}
```

## Placeholders

Backend commands support placeholders:

| Placeholder | Replaced With |
|-------------|---------------|
| `{packages}` | Space-separated package names |
| `{binary}` | The backend binary command |
| `{query}` | Search query string |

Example:
```kdl
install "pacman -S --needed {packages}"
search "{binary} -Ss {query}"
```

## Output Formats

For backend `list` commands, specify output format:

### whitespace
```kdl
list "pacman -Q" {
    format "whitespace"
    name_col 0
    version_col 1
}
```

### tsv (Tab-Separated)
```kdl
list "flatpak list" {
    format "tsv"
    name_col 0
}
```

### json
```kdl
list "npm list -g --json" {
    format "json"
    json {
        path "dependencies"
        name_key "name"
        version_key "version"
    }
}
```

### regex
```kdl
list "apt list --installed" {
    format "regex"
    regex {
        pattern "^([^/]+)/[^ ]+ ([^ ]+)"
        name_group 1
        version_group 2
    }
}
```

## Full Example

```kdl
// ~/.config/declarch/declarch.kdl
meta {
    title "Developer Workstation"
    author "johndoe"
    version "2.0.0"
}

imports {
    "backends.kdl"
    "modules/base.kdl"
    "modules/dev.kdl"
}

pkg {
    aur {
        neovim
        visual-studio-code-bin
    }
}

env "EDITOR" {
    nvim
}

on-sync "echo 'System updated'"
```

## Common Mistakes

### 1. Forgetting Quotes
```kdl
// Wrong
needs_sudo true
format whitespace

// Correct
needs_sudo "true"
format "whitespace"
```

### 2. Missing Closing Braces
```kdl
// Wrong
pkg {
    aur {
        neovim

// Correct
pkg {
    aur {
        neovim
    }
}
```

### 3. Wrong Block Structure
```kdl
// Wrong
pkg:aur {
    package
}

// Correct (aur inside pkg)
pkg {
    aur {
        package
    }
}
```

## Validation

Use `declarch check` to validate syntax:
```bash
declarch check
```

This will show line numbers for any errors.
