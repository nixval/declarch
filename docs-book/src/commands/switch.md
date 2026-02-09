# switch

Switch package variant.

## Usage

```bash
declarch switch [OPTIONS] <OLD_PACKAGE> <NEW_PACKAGE>
```

## Examples

### Switch Package

```bash
declarch switch neovim neovim-git
```

Replaces `neovim` with `neovim-git` in your config.

### Switch with Backend

```bash
declarch switch firefox aur:firefox-nightly
```

## How It Works

1. Removes old package from config
2. Adds new package to config
3. Syncs the changes

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | Specify backend |
| `--dry-run` | Preview only |

## Example

```bash
# Preview first
declarch switch firefox firefox-nightly --dry-run

# Then execute
declarch switch firefox firefox-nightly
```
