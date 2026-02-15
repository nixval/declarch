# Modules

Modules help keep config readable.

## Why modules?

Instead of one huge file, split by purpose:

```text
base.kdl
dev.kdl
work.kdl
gaming.kdl
```

## Create one

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

## Import it

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}
```

## Template

```kdl
meta {
    title "Dev"
    description "Development tools"
}

pkg {
    aur {
        // packages here
    }
}
```

## Practical tips

1. One module = one context.
2. Use obvious names.
3. Keep each module short.
