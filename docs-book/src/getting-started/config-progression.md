# Config Progression (Minimal -> Advanced)

This page shows a practical progression for you to start small and scale safely.

## Stage 1: Minimal

`declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
}
```

`modules/base.kdl`

```kdl
pkg {
    aur {
        git
        neovim
    }
}
```

Use this stage to validate your first workflow:

```bash
declarch --dry-run sync
declarch sync
```

## Stage 2: Add second backend

`modules/base.kdl`

```kdl
pkg {
    aur {
        git
        neovim
    }
    npm {
        typescript
        pnpm
    }
}
```

## Stage 3: Split by purpose (modular)

`declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/development.kdl"
}
```

`modules/development.kdl`

```kdl
pkg {
    cargo {
        ripgrep
        fd-find
    }
    npm {
        eslint
        prettier
    }
}
```

## Stage 4: Advanced targeting (profile/host)

Use profile/host blocks for machine-specific or role-specific packages.

Example concept:

```kdl
profile "work" {
    pkg {
        npm { @angular/cli }
    }
}

host "laptop-1" {
    pkg {
        flatpak { com.discordapp.Discord }
    }
}
```

Activate with:

```bash
declarch sync --profile work --host laptop-1
```

## Rule of thumb

- Keep `base.kdl` stable and small.
- Add new modules only when package groups become hard to scan.
- Introduce profile/host targeting only after base + modules are clear.
