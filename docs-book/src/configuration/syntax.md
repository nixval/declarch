# Syntax Reference (Advanced)

This page is intentionally technical.

If you are new to declarch, read these first:
- [Quick Start](../getting-started/quick-start.md)
- [KDL Basics](./kdl-syntax.md)
- [Backends](./backends.md)

---

## Purpose

This is the detailed reference for KDL structures, advanced keys, and edge cases.
Use it when you need precise behavior tuning.

## Core blocks

```kdl
meta { ... }
imports { ... }
pkg { ... }
```

## pkg block

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

## Backend-specific prefixes

```kdl
pkg {
    npm:typescript
    flatpak:org.mozilla.firefox
}
```

## Options, env, and repos overrides

```kdl
options:npm {
    noconfirm_flag "--yes"
}

env:global {
    "http_proxy=http://127.0.0.1:8080"
}

repos:pacman {
    "core"
    "extra"
}
```

## Policy

```kdl
policy {
    protected "linux" "systemd"
    orphans "ask"
}
```

## Hooks lifecycle

```kdl
hooks {
    pre-sync "echo before"
    post-sync "echo after"
}
```

## Notes

- Backend behavior can evolve with package-manager changes.
- Prefer simple config first; only use advanced blocks when needed.
