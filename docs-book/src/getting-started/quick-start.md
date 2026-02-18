# Quick Start

## 0) Understand the flow first (declarative)

Declarch works best when you treat KDL config as source of truth.

1. Declare packages in module files.
2. Preview with dry-run.
3. Apply with sync.

```bash
declarch --dry-run sync
declarch sync
```

## 1) Initialize

```bash
declarch init
```

Expected structure:

```text
~/.config/declarch/
├── declarch.kdl
├── backends/
└── modules/
    └── base.kdl
```

Use this anytime to see actual config/state paths on your OS:

```bash
declarch info --doctor -v
```

## 2) Adopt backend(s) first

```bash
# Arch-based
declarch init --backend aur,paru,yay,pacman

# Debian/Ubuntu
declarch init --backend nala,apt

# Fedora/RHEL
declarch init --backend dnf5

# SUSE
declarch init --backend zypper

# macOS
declarch init --backend brew

# discover options from registry
declarch init --list backends
declarch init --list modules
```

## 3) Add packages (declarative first)

Example module file (`~/.config/declarch/modules/mydotfiles.kdl`):

```kdl
meta {
    title "My dotfiles"
}

pkg {
    aur { hyprland waybar }
    flatpak { firefox }
    soar { gimp }
    npm {
        @opencode-ai/sdk
        oh-my-opencode
    }
}
```

Edit flow:

```bash
declarch edit
declarch edit mydotfiles --create
declarch edit mydotfiles
```

Then:

```bash
declarch --dry-run sync
declarch lint --fix
declarch sync prune
declarch sync
```

## 4) Direct install (optional shortcut)

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
declarch install bat fzf ripgrep --backend aur
```

This writes package entries to `modules/others.kdl` automatically.

## 5) Add more backends when needed

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid
declarch init --backend pnpm yarn
```

That is the core workflow.
