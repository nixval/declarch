# Modules

Modules let you split your config into multiple files.

## Why use modules?

Instead of one giant file, split by purpose:

```text
base.kdl      # Essentials
dev.kdl       # Development
gaming.kdl    # Games
work.kdl      # Work
```

## Create a module

```bash
mkdir -p ~/.config/declarch/modules
cat > ~/.config/declarch/modules/dev.kdl << 'EOKDL'
pkg {
    aur {
        neovim
        tmux
    }

    npm {
        typescript
    }
}
EOKDL
```

## Import modules

Add to `~/.config/declarch/declarch.kdl`:

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

## Module template

```kdl
// modules/example.kdl
meta {
    title "Example Module"
    description "What this module contains"
}

pkg {
    aur {
        // packages here
    }
}
```

## Best practices

1. Keep one module for one purpose.
2. Use clear names (`gaming.kdl`, not `misc.kdl`).
3. Add short metadata so future-you remembers intent.

## Disable module quickly

```kdl
imports {
    "modules/base.kdl"
    // "modules/gaming.kdl"
}
```
