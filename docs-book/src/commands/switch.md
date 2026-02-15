# switch

Replace one package entry with another.

## Usage

```bash
declarch switch [OPTIONS] <OLD_PACKAGE> <NEW_PACKAGE>
```

## Examples

```bash
declarch switch neovim neovim-git
declarch switch firefox aur:firefox-nightly

# preview
declarch switch firefox firefox-nightly --dry-run
```

## What it does

1. removes old package from config
2. adds new package to config
3. syncs the change

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | backend scope |
| `--dry-run` | preview only |
