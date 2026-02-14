# Quick Start

A practical first run.

## 1) Initialize

```bash
declarch init
```

You will get:

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── state.json
├── backends/
└── modules/
    └── base.kdl
```

## 2) Add packages

```bash
declarch install bat fzf ripgrep
```

Optional: explicit backend

```bash
declarch install npm:typescript
```

## 3) Sync

```bash
declarch sync
```

Core loop:
- edit/add packages
- sync

## 4) Preview before applying

```bash
declarch sync preview
```

Useful when learning or migrating.

## 5) Add more backends when needed

```bash
declarch init --backend npm
# or
declarch init --backend pnpm,yarn
```

You can mix comma and space style safely.
