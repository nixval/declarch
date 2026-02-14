# declarch

Declarative package management, without trying to be magical.

You describe packages in config files, then run `sync` to make your system follow that config.

## WARNING: v0.8.0 has BREAKING CHANGES

If you are upgrading from older versions, expect config and workflow changes.

What changed in plain words:
- Backend architecture changed.
- Some CLI flows changed.
- KDL syntax expectations are stricter in multiple places.

Please back up your current config before migrating:

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
```

Important note:
- declarch is actively evolving.
- Not every backend/environment combination has been fully tested yet.
- If something feels off, check docs/troubleshooting first and then open an issue.

## What declarch is (and is not)

declarch is:
- A declarative layer on top of multiple package backends.
- Useful if you want reproducible package setup across machines.
- Modular: split config by host/use-case.

declarch is not:
- A guaranteed universal abstraction for every backend edge case.
- A promise that all backend commands behave identically everywhere.

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

### 1. Initialize config

```bash
declarch init
```

This creates:

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
└── modules/
    └── base.kdl
```

### 2. Add packages

```bash
declarch install bat fzf ripgrep
```

With explicit backend:

```bash
declarch install npm:typescript
```

### 3. Apply changes

```bash
declarch sync
```

## Basic configuration example

Edit `~/.config/declarch/modules/base.kdl`:

```kdl
pkg {
    pacman {
        firefox
        tmux
    }

    flatpak {
        org.mozilla.firefox
    }

    npm {
        typescript
        pnpm
    }
}
```

Then:

```bash
declarch sync
```

## Common commands

```bash
# Preview changes
declarch sync preview

# Sync + backend updates
declarch sync update

# Remove unmanaged packages
declarch sync prune

# Search
declarch search firefox

# Validate config
declarch check

# System/state info
declarch info
```

## Backend setup

To adopt extra backends from registry:

```bash
declarch init --backend npm
```

Multiple backends (space-separated):

```bash
declarch init --backend pnpm yarn
```

Multiple backends (comma-separated):

```bash
declarch init --backend pnpm,yarn
```

Use `--force` to overwrite existing backend file.

## Documentation

Hosted docs:
- https://nixval.github.io/declarch/

mdDocs source pages in this repo (`docs-book/src`):
- [Getting Started](docs-book/src/getting-started/quick-start.md)
- [Installation](docs-book/src/getting-started/installation.md)
- [Commands Overview](docs-book/src/commands/index.md)
- [init command](docs-book/src/commands/init.md)
- [sync command](docs-book/src/commands/sync.md)
- [Backends config](docs-book/src/configuration/backends.md)
- [KDL syntax](docs-book/src/configuration/kdl-syntax.md)
- [Troubleshooting](docs-book/src/advanced/troubleshooting.md)

Full docs structure:
- [SUMMARY](docs-book/src/SUMMARY.md)

## License

MIT — see `LICENSE`.
