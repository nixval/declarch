# Basic Concepts

This page is beginner-first.

## 1) Declarch is a wrapper

Declarch orchestrates real package managers.
You are still using actual tools under the hood (`apt`, `pacman`, `flatpak`, `npm`, `nix`, etc).

## 2) Agnostic design

You can mix ecosystems in one config:

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

## 3) Declarative loop

1. declare packages
2. run sync

```bash
declarch sync
```

## 4) Backend configs can evolve

Package managers change over time.
Declarch backend definitions are designed to be editable and update-friendly.

## 5) Modules keep things simple

Use small files by context:
- `base.kdl`
- `dev.kdl`
- `work.kdl`
- `gaming.kdl`

## 6) Start safe

When unsure, use:

```bash
declarch sync preview
```
