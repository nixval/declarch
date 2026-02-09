# Modules

Modules let you split your config into multiple files.

## Why Use Modules?

Instead of one giant file:

```
base.kdl      # 200 lines
```

Organize logically:

```
base.kdl      # Essentials (20 lines)
dev.kdl       # Development (30 lines)
gaming.kdl    # Games (15 lines)
work.kdl      # Work stuff (25 lines)
```

## Creating a Module

```bash
# Create the file
mkdir -p ~/.config/declarch/modules
cat > ~/.config/declarch/modules/dev.kdl << 'EOF'
pkg {
    aur {
        neovim
        tmux
    }
    
    npm {
        typescript
    }
}
EOF
```

## Importing Modules

Add to `~/.config/declarch/declarch.kdl`:

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

## Module Template

```kdl
// modules/example.kdl
meta {
    title "Example Module"
    description "What this module contains"
}

pkg {
    aur {
        # packages here
    }
}
```

## Best Practices

1. **Keep it focused** - One module = one purpose
2. **Use descriptive names** - `gaming.kdl` not `stuff.kdl`
3. **Add metadata** - Helps you remember what's in it

## Example Structure

```
modules/
├── base.kdl        # Essentials: bat, fzf, git
├── dev.kdl         # Dev tools: neovim, docker
├── desktop.kdl     # GUI apps: firefox, slack
└── gaming.kdl      # Steam, Lutris, etc.
```

## Disabling Modules

Comment out the import:

```kdl
imports {
    "modules/base.kdl"
    // "modules/gaming.kdl"  # Disabled
}
```
