# declarch

Declarch is a declarative wrapper for multiple package managers.

It can orchestrate many package managers/helpers through configuration.
In declarch terminology, we call those integrations **backends**.

You declare packages in KDL files, then apply with `declarch sync`.

## WARNING: v0.8.0 has BREAKING CHANGES

If you are upgrading from older versions, expect syntax and workflow changes.
This warning matters most if your config was created before the v0.8 line.

Before upgrading:

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
```

To check your actual config/state paths on your OS:

```bash
declarch info --doctor
```

## Core flow (must understand)

This is the daily loop most users follow. Keep it simple:

1. Declare packages in config.
2. Preview changes.
3. Apply.

```bash
declarch --dry-run sync
declarch sync
```

## Installation

Pick one installation path first, then verify CLI + doctor output.

### Arch Linux (AUR)

Best path if you are already on Arch and use AUR helpers.

```bash
paru -S declarch
# or
yay -S declarch
```

### Linux/macOS (install script)

Fastest way to get a ready binary from GitHub releases.

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### Windows (PowerShell, experimental alpha)

Windows support is still evolving, but this gives you a quick first install.

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Verify installation:

```bash
declarch --version
declarch --help
declarch info --doctor -v
```

## First setup (beginner path)

After install, this is the shortest path to a working declarative setup.

### 1) Initialize config

This creates the base config structure and default entrypoints.

```bash
declarch init
```

### 2) Adopt backends for your OS

Backends define how declarch talks to each package manager/helper.

```bash
# Arch
declarch init --backend aur,paru,yay,pacman

# Debian/Ubuntu
declarch init --backend apt,nala

# Fedora/RHEL
declarch init --backend dnf5

# SUSE
declarch init --backend zypper

# macOS
declarch init --backend brew

# discover registry options
declarch init --list backends
declarch init --list modules
```

Common backend examples you can `init`:
- System/distro: `pacman`, `aur`, `paru`, `yay`, `apt`, `nala`, `dnf`, `dnf5`, `zypper`, `brew`
- Universal/app: `flatpak`, `snap`, `nix`, `soar`
- Dev/language: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`

### 3) Add packages (declarative-first)

This is the core value: one config can drive multiple ecosystems.

Example `~/.config/declarch/modules/base.kdl`:

```kdl
pkg {
    aur { bat fzf ripgrep }
    npm { typescript }
}
```

Then run:

```bash
declarch --dry-run sync
declarch sync
```

### 4) Install shortcut (optional)

Use this when you want fast onboarding; you can refactor into modules later.

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
# or apply one backend to all packages
declarch install bat fzf ripgrep --backend aur
```

## Configuration examples

If you are unsure where to start, copy one of these and iterate.

### A) Minimal portable setup

Good default for a single machine or fresh setup.

`~/.config/declarch/declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
}
```

`~/.config/declarch/modules/base.kdl`

```kdl
pkg {
    pacman { git curl }
    flatpak { org.mozilla.firefox }
    npm { pnpm }
}
```

### B) Modular setup by context

Use this when your config starts growing (dev/apps/workstation split).

`declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/apps.kdl"
}
```

`modules/dev.kdl`

```kdl
pkg {
    aur { neovim tmux }
    npm { typescript eslint prettier }
}
```

`modules/apps.kdl`

```kdl
pkg {
    flatpak { org.telegram.desktop com.spotify.Client }
}
```

For advanced topics (policy, hooks, profile/host layering, MCP, and deeper syntax), use the docs site.

## Most-used commands

These cover most day-to-day workflows without touching advanced features.

```bash
declarch sync
declarch --dry-run sync
declarch sync update
declarch sync prune
declarch sync cache
declarch sync upgrade
declarch search firefox
declarch info --doctor
declarch info --list --scope unmanaged
declarch lint --mode validate
declarch edit mymodule --create
```

## Update policy

Use the update path that matches how you installed declarch.

- If installed via package manager (AUR/Homebrew/etc), update via that package manager.
- If installed via script/manual binary, you can use `declarch self-update`.

## Documentation

- https://nixval.githu.io/declarch/

## License

MIT - see `LICENSE`.
