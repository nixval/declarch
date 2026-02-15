# install

Add packages to config quickly.

## Usage

```bash
declarch install [OPTIONS] <PACKAGES>...
```

## Common examples

```bash
declarch install neovim
declarch install bat fzf ripgrep

declarch install npm:typescript
declarch install aur:neovim

declarch install -b flatpak org.mozilla.firefox
declarch install --module browsers firefox
```

## What happens

1. Package entries are written to a module (`modules/others.kdl` by default).
2. `declarch sync` runs automatically, unless `--no-sync` is used.

If you do not pass backend, declarch selects distro-aware default backend.

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | force backend for all packages |
| `-m, --module <NAME>` | target module file |
| `--no-sync` | edit only, skip sync |
