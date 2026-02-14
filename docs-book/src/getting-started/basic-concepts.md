# Basic Concepts

This page is for first-time users.
No advanced internals here.

## 1. Declarch is a wrapper

Declarch wraps multiple package managers in one workflow.
You still use real backends under the hood (`pacman`, `flatpak`, `npm`, `nix`, etc).

## 2. Agnostic by design

You can mix backends in one config:

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

## 3. Declarative workflow

You declare desired state, then sync:

```bash
declarch sync
```

Declarch compares config vs system and applies only what is needed.

## 4. Backend configs can evolve

Package managers change over time.
Declarch backend definitions are intentionally flexible so command templates/format parsers can be updated without redesigning your whole workflow.

## 5. Keep modules simple

Split by context:
- `base.kdl`
- `dev.kdl`
- `work.kdl`
- `gaming.kdl`

This is easier to maintain than one giant file.

## 6. Be realistic

Declarch works well for common flows, but not every backend + environment combo is tested equally.
When something fails, check backend config and troubleshooting docs.
