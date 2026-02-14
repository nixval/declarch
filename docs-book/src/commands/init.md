# init

Create initial configuration, adopt backends, or fetch remote config.

## Usage

```bash
declarch init [OPTIONS] [SOURCE]
```

## Common examples

### Basic init

```bash
declarch init
```

Creates:

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── state.json
├── backends/
└── modules/
    └── base.kdl
```

### Add backend(s)

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid
declarch init --backend pnpm yarn
```

### List available remote backends

```bash
declarch init --list backends
```

### Initialize from remote source

```bash
declarch init username/dotfiles
declarch init username/dotfiles:minimal
```

## Options

| Option | Description |
|--------|-------------|
| `--backend <NAME>...` | Add one or more backend definitions |
| `--list <WHAT>` | List available `backends` or `modules` |
| `--local` | Create module locally (skip registry lookup) |
| `--host <NAME>` | Set hostname for root template |
| `--restore-backends` | Restore `backends.kdl` from template |
| `--restore-declarch` | Restore `declarch.kdl` from template |

## Official defaults and custom backends

`declarch init` gives official defaults in `backends.kdl` (`aur`, `pacman`, `flatpak`, plus shipped defaults).

`declarch init --backend <name>` adopts backend definitions from remote registry into your local `backends/` folder.

If a backend already exists locally, use `--force` to overwrite.
