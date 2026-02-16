# declarch

Declarch is a declarative, agnostic wrapper for many package managers.

You write package config once, then run `declarch sync`.
Declarch handles the backend commands for you.

## WARNING: v0.8.0 has BREAKING CHANGES

If you are upgrading from older versions, expect syntax and workflow changes.

Before upgrading:

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
```

Linux example shown above. For your exact paths on macOS/Windows, run:

```bash
declarch info --doctor
```

Reality check:
- declarch is still evolving,
- backend/environment coverage keeps improving,
- not every backend combo is tested equally yet.

Use `declarch --dry-run sync` first when unsure.

## What declarch is

- **Wrapper** for existing package managers.
- **Agnostic** architecture (not locked to one ecosystem).
- **Flexible backend config** that can evolve with upstream tools.

## Common backends

I call all of these "backends" in declarch (package manager, helper, wrapper, and similar tools).

You can use built-in/default ones, fetch extra backend definitions from `nixval/declarch-packages`, or create your own declaratively.

Discover available backends:

```bash
declarch init --list backends
```

Adopt one:

```bash
declarch init --backend <backend-name>
```

Examples:
`aur`, `pacman`, `flatpak`, `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`, `nix`, `apt`, `nala`, `dnf`, `snap`, `brew`, `soar`, and more.

Planned Windows backend set (experimental roadmap): `winget`, `choco`, `scoop`.

Declarch started with strong Arch focus, but the same declarative pattern works for many backends.
So you do not need to remember dozens of rarely-used commands.

Common flow stays simple:
`declarch sync`, `declarch sync prune`, `declarch sync update`, `declarch sync upgrade`, `declarch search`, `declarch info`, `declarch info --list`.

## Basic config example

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript pnpm }
    nix { nil }
}
```

Then run:

```bash
declarch sync
```

Packages will be installed or adopted when backend is available.
If backend is missing, declarch will skip it with warning.

## Installation

### Arch Linux (AUR)

```bash
paru -S declarch
```

### Script install

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

macOS support via script is currently **experimental (alpha)**.
Installer includes lightweight smoke checks after install (`--help`, `info`).

### Windows (PowerShell, experimental alpha)

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Installer includes lightweight smoke checks after install (`--help`, `info`).

## First-time setup

### 1. Initialize

```bash
declarch init
```

Default config includes ready backend definitions (`aur`, `pacman`, `flatpak`, and shipped defaults).
Add more anytime with:

```bash
declarch init --backend <backend-name>
```

### 2. Add packages

`install` now requires explicit backend, either per package or via flag.

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
# or
declarch install bat fzf ripgrep --backend aur
```

### 3. Apply

```bash
declarch sync
```

Use preview when needed:

```bash
declarch --dry-run sync
```

## Backend setup

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid:
declarch init --backend pnpm yarn
```

Use `--force` to overwrite an existing backend file.

## Common commands

```bash
declarch sync
declarch --dry-run sync
declarch sync update
declarch sync prune
declarch search firefox
declarch lint
declarch info
declarch info --list
```

Machine-readable placeholder contract (v1, staged rollout):

```bash
declarch info --format json --output-version v1
declarch info --list --format yaml --output-version v1
declarch lint --format json --output-version v1
declarch search firefox --format json --output-version v1
declarch --dry-run sync --format json --output-version v1
```

## Documentation

Hosted docs:
- https://nixval.github.io/declarch/

mdDocs source pages (`docs-book/src`):
- [Introduction](docs-book/src/intro.md)
- [Installation](docs-book/src/getting-started/installation.md)
- [Quick Start](docs-book/src/getting-started/quick-start.md)
- [First Run (Linear Guide)](docs-book/src/getting-started/first-run-linear.md)
- [Common Mistakes](docs-book/src/getting-started/common-mistakes.md)
- [Config Progression](docs-book/src/getting-started/config-progression.md)
- [Cross-OS (Alpha)](docs-book/src/getting-started/cross-os-alpha.md)
- [Command Overview](docs-book/src/commands/index.md)
- [Backends](docs-book/src/configuration/backends.md)
- [KDL Basics](docs-book/src/configuration/kdl-syntax.md)
- [Syntax Reference (Advanced)](docs-book/src/configuration/syntax.md)
- [Integration Roadmap RFC](docs-book/src/advanced/rfc-integration-roadmap.md)
- [Integration Examples](docs-book/src/advanced/integration-examples.md)
- [Troubleshooting](docs-book/src/advanced/troubleshooting.md)
- [Full sidebar](docs-book/src/SUMMARY.md)

## License

MIT - see `LICENSE`.
