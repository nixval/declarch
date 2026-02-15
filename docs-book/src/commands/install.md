# install

Add packages to config quickly.

## Usage

```bash
declarch install [OPTIONS] <PACKAGES>...
```

## Important

`install` requires explicit backend now.

Use one of these styles:
- `backend:package` per package
- `--backend <name>` for all packages

## Common examples

```bash
declarch install aur:neovim
declarch install aur:bat aur:fzf aur:ripgrep

declarch install npm:typescript

declarch install org.mozilla.firefox --backend flatpak
declarch install firefox --module browsers --backend aur
```

## What happens

1. Package entries are written to a module (`modules/others.kdl` by default).
2. `declarch sync` runs automatically, unless `--no-sync` is used.
3. If sync is cancelled or fails, config changes are rolled back.

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | force backend for all packages |
| `-m, --module <NAME>` | target module file |
| `--no-sync` | edit only, skip sync |
