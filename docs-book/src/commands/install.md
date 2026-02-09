# install

Add packages to configuration.

## Usage

```bash
declarch install [OPTIONS] <PACKAGES>...
```

## Examples

### Install Packages

```bash
# Single package
declarch install neovim

# Multiple packages
declarch install bat fzf ripgrep fd
```

### Specify Backend

```bash
# With backend prefix
declarch install aur:neovim
declarch install npm:typescript
```

### Target Module

```bash
# Add to specific module
declarch install firefox --module browsers
```

## How It Works

1. Adds packages to `modules/others.kdl` (or specified module)
2. Runs `declarch sync` (unless `--no-sync`)

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | Backend for all packages |
| `-m, --module <NAME>` | Target module |
| `--no-sync` | Don't sync after adding |

## Examples

```bash
# Add to default module
declarch install neovim

# Add to specific backend
declarch install -b aur neovim

# Add to specific module
declarch install -m dev docker

# Add only (don't sync)
declarch install neovim --no-sync
```

## Default Backend

If no backend specified, packages are added to the `aur` block.
