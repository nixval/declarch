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

That is the core loop:
- edit/add packages
- sync

## 4) Preview before applying

```bash
declarch sync preview
```

Useful when you are still learning or doing migration.

## 5) Add more backends when needed

```bash
declarch init --backend npm
# or
declarch init --backend pnpm,yarn
```

You can mix space and comma style safely.
