# Introduction

Declarch is a declarative wrapper for multiple package managers.

In short:
- you write what packages you want,
- declarch picks the right backend,
- then `declarch sync` applies it.

## What makes it different

Declarch is:
- **Agnostic**: it does not lock you into one package manager.
- **Wrapper-style**: it orchestrates existing tools (`pacman`, `aur`, `flatpak`, `npm`, `nix`, etc).
- **Flexible**: backend configs can evolve as upstream package managers evolve.

Declarch is **not**:
- a replacement for every backend CLI feature,
- a promise that every backend/environment combo is perfectly tested yet.

## Backends you can use

Common backends include:
- `aur`, `pacman`, `flatpak`
- `npm`, `pnpm`, `yarn`, `bun`
- `cargo`, `pip`, `gem`, `go`
- `apt`, `nala`, `dnf`, `snap`, `brew`, `nix`, `soar`

## 60-second flow

```bash
# 1) initialize
declarch init

# 2) add packages
declarch install bat ripgrep

# 3) apply
declarch sync
```

## Important note for v0.8.0+

v0.8.0 introduced breaking changes in architecture and syntax.
If you are upgrading from older versions, back up config first and follow docs step-by-step.

Next:
- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)
