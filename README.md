# declarch

Declarch is a declarative, agnostic wrapper over many package managers.

You define packages in config, then run `declarch sync`.
Declarch coordinates backend commands for you.

## WARNING: v0.8.0 has BREAKING CHANGES

If you are upgrading from older versions, expect syntax and workflow changes.

Before upgrading:

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
```

Reality check:
- declarch is still evolving,
- backend/environment coverage is improving,
- not every backend combination is tested equally yet.

Use `declarch sync preview` first when unsure.

## What declarch is

- **Wrapper** for existing package managers.
- **Agnostic** architecture (not tied to one ecosystem).
- **Flexible backend config** that can evolve with upstream tools.

## Common backends

`aur`, `pacman`, `flatpak`, `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`, `nix`, `apt`, `nala`, `dnf`, `snap`, `brew`, `soar`.

## Installation

### Arch Linux (AUR)

```bash
paru -S declarch
```

### Script install

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

## First-time setup

### 1. Initialize

```bash
declarch init
```

### 2. Add packages

```bash
declarch install bat fzf ripgrep
declarch install npm:typescript
```

### 3. Apply

```bash
declarch sync
```

Use preview when needed:

```bash
declarch sync preview
```

## Basic config example

```kdl
pkg {
    pacman { firefox git }
    flatpak { org.mozilla.firefox }
    npm { typescript pnpm }
    nix { nil }
}
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
declarch sync preview
declarch sync update
declarch sync prune
declarch search firefox
declarch check
declarch info
declarch info list
```

## Documentation

Hosted docs:
- https://nixval.github.io/declarch/

mdDocs source pages (`docs-book/src`):
- [Introduction](docs-book/src/intro.md)
- [Installation](docs-book/src/getting-started/installation.md)
- [Quick Start](docs-book/src/getting-started/quick-start.md)
- [Command Overview](docs-book/src/commands/index.md)
- [Backends](docs-book/src/configuration/backends.md)
- [KDL Basics](docs-book/src/configuration/kdl-syntax.md)
- [Syntax Reference (Advanced)](docs-book/src/configuration/syntax.md)
- [Troubleshooting](docs-book/src/advanced/troubleshooting.md)
- [Full sidebar](docs-book/src/SUMMARY.md)

## License

MIT - see `LICENSE`.
