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

## That's It

3 packages, 2 files. Run `declarch sync` and you're done.

## Why This Works

- `backends.kdl` already has aur/pacman/flatpak built-in
- No custom backends needed
- Single module keeps it simple

## Next Steps

Add more packages:
```bash
declarch install bat fzf ripgrep
```
