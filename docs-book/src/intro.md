# Introduction

Declarch is a declarative wrapper for many package managers.

Short version:
- you list what you want,
- declarch maps it to backends,
- `declarch sync` applies it.

## Why people use it

- One workflow for mixed ecosystems (`pacman`, `flatpak`, `npm`, `nix`, etc).
- Config-first style that is easier to review and version.
- Backend configs are flexible, so they can follow upstream package-manager changes.

## What it is not

- Not a replacement for every backend CLI feature.
- Not guaranteed perfect on every backend/environment pair yet.

## v0.8 note

v0.8 introduced breaking changes.
If you are migrating, back up config first and start with preview.

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
declarch sync preview
```

Path above is Linux-style. Use this to print real paths on your OS:

```bash
declarch info --doctor
```

## One important behavior

`declarch install` expects explicit backend now.
Use `backend:package` or `--backend <name>`.

## Start here

1. [Installation](./getting-started/installation.md)
2. [Quick Start](./getting-started/quick-start.md)
3. [Basic Concepts](./getting-started/basic-concepts.md)
