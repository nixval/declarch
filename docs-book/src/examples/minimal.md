# Minimal Setup

The simplest possible declarch configuration.

## Files

### declarch.kdl

```kdl
imports {
    "modules/base.kdl"
}
```

### modules/base.kdl

```kdl
pkg {
    aur {
        neovim
        git
    }
}
```

## That's it

Two files, a few packages:

```bash
declarch sync
```

## Why this works

- Root config and default backend definitions are created by `declarch init`.
- You can add/adopt extra backends later with `declarch init --backend <name>`.
- Keeping one small module is easiest for beginners.

## Next step

```bash
declarch install bat fzf ripgrep
```
