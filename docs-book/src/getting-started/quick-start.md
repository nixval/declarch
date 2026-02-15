# Quick Start

Fast first run.

## 1) Initialize

```bash
declarch init
```

Expected structure:

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── state.json
├── backends/
└── modules/
    └── base.kdl
```

## 2) Add packages (explicit backend required)

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript

# or same backend for all
declarch install bat fzf ripgrep --backend aur
```

## 3) Preview and apply

```bash
declarch sync preview
declarch sync
```

## 4) Add more backends only when needed

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid
declarch init --backend pnpm yarn
```

That is the core workflow.
