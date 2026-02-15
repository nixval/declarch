# Development Example

Beginner-friendly dev setup with separate modules.

## Prerequisite

```bash
declarch init --backend npm
```

## Structure

```text
modules/
├── base.kdl
├── dev.kdl
└── langs.kdl
```

## `modules/dev.kdl`

```kdl
pkg {
    aur {
        neovim
        tmux
        docker
    }

    pacman {
        git
        github-cli
        jq
    }
}
```

## `modules/langs.kdl`

```kdl
pkg {
    aur {
        rustup
    }

    npm {
        typescript
        ts-node
        prettier
        eslint
    }
}
```

## `declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/langs.kdl"
}
```

## Apply

```bash
declarch sync preview
declarch sync
```
