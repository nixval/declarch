# KDL Syntax

Declarch uses KDL ("Kay-dee-el") - a simple config format.

## Basic Structure

```kdl
// This is a comment

pkg {
    aur {
        neovim
        bat
    }
}
```

## Key Rules

### 1. Everything Quoted

All string values must be in quotes:

```kdl
// ✓ Correct
format "whitespace"
needs_sudo "true"

// ✗ Wrong
format whitespace
needs_sudo true
```

### 2. Blocks Use Braces

```kdl
pkg {
    aur {
        neovim
    }
}
```

### 3. Comments Use `//`

```kdl
// This is a comment
aur {
    neovim  // This is an inline comment
}
```

## Common Patterns

### Package Blocks

```kdl
pkg {
    aur {
        neovim
        bat
        fzf
    }
    
    flatpak {
        com.spotify.Client
    }
}
```

### Metadata

```kdl
meta {
    title "My Setup"
    description "Development workstation"
}
```

### Imports

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

## Complete Example

```kdl
// declarch.kdl
meta {
    title "Workstation"
    author "you"
}

imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}

pkg {
    aur {
        neovim
    }
}
```

## Error Messages

Declarch shows helpful errors:

```
error: No closing '}' for child block
  --> ~/.config/declarch/base.kdl:8:5
   |
 6 │     aur {
 7 │         neovim
 8 │     meta {
   │      ^
   │
```

## Learn More

- [KDL Official Spec](https://kdl.dev/)
