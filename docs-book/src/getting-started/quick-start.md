# Quick Start

Fast first run.

If you want a strict step-by-step guide, use:
- [First Run (Linear Guide)](./first-run-linear.md)
- [Common Mistakes](./common-mistakes.md)

## 1) Initialize

```bash
declarch init
```

Expected structure (Linux example):

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── backends/
└── modules/
    └── base.kdl
```

State file is stored separately in the OS state directory.
Linux example:

```text
~/.local/state/declarch/state.json
```

Use this anytime to see actual config/state paths on your OS:

```bash
declarch info --doctor
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
declarch --dry-run sync
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
