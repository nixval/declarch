# Declarch Configuration Examples

This directory contains simple examples aligned with current `v0.8.x` docs.

## File structure

```text
examples/
├── declarch.kdl
├── backends.kdl
├── modules/
│   └── base.kdl
├── minimal.kdl
├── modular.kdl
├── desktop.kdl
└── development.kdl
```

## Quick start

1. Copy files:

```bash
mkdir -p ~/.config/declarch/modules
cp examples/declarch.kdl ~/.config/declarch/
cp examples/modules/base.kdl ~/.config/declarch/modules/
cp examples/backends/backends.kdl ~/.config/declarch/
```

2. Adjust backend defs for your machine.

3. Apply:

```bash
declarch sync preview
declarch sync
```

## Package syntax (recommended)

Use nested `pkg` blocks as default style:

```kdl
pkg {
    apt {
        vim
        git
    }
    flatpak {
        org.mozilla.firefox
    }
}
```

Legacy syntax may still parse for migration, but docs/examples prefer the nested style above.

## Backend config examples

`examples/backends/backends.kdl` demonstrates:

- distro/system backends (`apt`, `aur`, `pacman`, `flatpak`)
- language backends (`npm`, `cargo`)
- fallback patterns and parser formats

Use it as reference, then keep only what you need.
