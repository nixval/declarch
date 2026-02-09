# init

Create initial configuration.

## Usage

```bash
declarch init [OPTIONS] [SOURCE]
```

## Examples

### Basic Init

```bash
declarch init
```

Creates:
```
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── modules/
│   └── base.kdl
└── state.json
```

### With Backend

```bash
# Add npm backend
declarch init --backend npm

# Add multiple backends
declarch init --backend npm,cargo
```

### List Available Backends

```bash
declarch init --list backends
```

### From Remote Config

```bash
# From GitHub repo
declarch init username/dotfiles

# From specific variant
declarch init username/dotfiles:minimal
```

## Options

| Option | Description |
|--------|-------------|
| `--backend <NAMES>` | Add backend(s) |
| `--list <what>` | List available backends/modules |
| `--local` | Create local module (skip registry) |
| `--host <NAME>` | Hostname specific config |

## Built-in Backends

These are included automatically:

- `aur` - AUR packages (uses paru/yay)
- `pacman` - Official Arch repos
- `flatpak` - Flatpak apps

## Custom Backends

Install additional backends:

```bash
declarch init --backend npm     # Node.js
declarch init --backend cargo   # Rust
declarch init --backend pip     # Python
```
