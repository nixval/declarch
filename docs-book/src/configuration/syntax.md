# Syntax Reference (Advanced)

Technical reference for KDL structures used by declarch.

If you are new, read:
- [Quick Start](../getting-started/quick-start.md)
- [KDL Basics](./kdl-syntax.md)
- [Backends](./backends.md)

## Core blocks

```kdl
meta { ... }
imports { ... }
pkg { ... }
```

## Package declarations

### Preferred (nested)

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

### Also accepted (compatibility)

```kdl
pkg:pacman { firefox git }

pkg {
    pacman:firefox
    npm:typescript
}
```

## Optional advanced blocks

### Backend options override

```kdl
options:pacman {
    noconfirm_flag "--noconfirm"
}
```

### Env override

```kdl
env:global {
    "http_proxy=http://127.0.0.1:8080"
}

env:npm {
    "NPM_CONFIG_REGISTRY=https://registry.npmjs.org"
}
```

### Package source overrides (backend-specific)

```kdl
repos:pacman {
    "core"
    "extra"
}
```

### Hooks

```kdl
hooks {
    pre-sync "echo before"
    post-sync "echo after"
}
```

To execute hooks, users must opt in:

```kdl
experimental {
    "dangerously-enable-hooks"
}
```

And run sync with `--hooks`.

### Policy

```kdl
policy {
    protected "linux" "systemd"
    orphans "ask"
}
```

### Profile and host blocks

These are optional overlays and are inactive by default.

```kdl
profile "desktop" {
    pkg { aur { hyprland waybar } }
}

host "vps-1" {
    pkg { aur { fail2ban } }
}
```

Use from CLI:
- `declarch sync sync --profile desktop`
- `declarch sync sync --host vps-1`

## Validation notes

- Keep beginner config in nested `pkg` style unless migration requires compatibility syntax.
- Unknown keys may be ignored in some contexts for forward compatibility.
- Use `declarch check validate` after manual edits.
