# Minimal Setup

Smallest useful setup for beginners.

## `declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
}
```

## `modules/base.kdl`

```kdl
pkg {
    aur {
        neovim
        git
    }
}
```

## Apply

```bash
declarch --dry-run sync
declarch sync
```

## Why this is a good start

- only one module
- easy to read
- easy to expand later
