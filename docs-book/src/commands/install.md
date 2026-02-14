# install

Add packages to configuration.

## Usage

```bash
declarch install [OPTIONS] <PACKAGES>...
```

## Examples

```bash
# Single package
declarch install neovim

# Multiple packages
declarch install bat fzf ripgrep fd

# Explicit backend prefix
declarch install aur:neovim
declarch install npm:typescript

# Force one backend for all packages
declarch install -b flatpak org.mozilla.firefox

# Target module
declarch install firefox --module browsers
```

## How it works

1. Adds package entries into a module file (`modules/others.kdl` by default).
2. Auto-runs `declarch sync` unless `--no-sync` is used.

If backend is not specified, declarch picks distro-aware default backend (`aur`, `apt`, `dnf`, or fallback logic).

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | Backend for all packages |
| `-m, --module <NAME>` | Target module |
| `--no-sync` | Skip auto sync |
