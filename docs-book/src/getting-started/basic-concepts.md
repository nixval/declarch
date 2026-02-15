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

## 4) Keep config portable first

If you copy your config to another machine (laptop, VPS, etc), it should still work with default sync:

```bash
declarch sync
```

No `host` or `profile` required.

## 5) Backend configs can evolve

Package managers change over time.
Declarch backend definitions are designed to be editable and update-friendly.

## 6) Modules keep things simple

Use small files by context:
- `base.kdl`
- `dev.kdl`
- `work.kdl`
- `gaming.kdl`

## 7) Optional profile/host layering

If you need machine-specific extras, you can opt in:

```bash
declarch sync sync --profile desktop
declarch sync sync --host vps-1
```

If you do not pass those flags, declarch uses your normal default config only.

## 8) Start safe

When unsure, use:

```bash
declarch sync preview
```
