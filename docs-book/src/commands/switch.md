# declarch switch

Replace one package with another.

## Usage

```bash
declarch switch <OLD_PACKAGE> <NEW_PACKAGE>
```

## Quick Start

```bash
# Switch to git version
declarch switch hyprland hyprland-git

# Preview first
declarch switch firefox firefox-beta --dry-run
```

## What It Does

1. Removes old package from config
2. Adds new package to config
3. Installs new package
4. Removes old package

## Common Uses

### Stable → Git Version

```bash
declarch switch hyprland hyprland-git
declarch switch waybar waybar-git
```

### Test Beta Versions

```bash
declarch switch firefox firefox-beta
declarch switch neovim neovim-nightly
```

### Replace Deprecated Packages

```bash
declarch switch exa eza
```

### Switch Back

```bash
declarch switch hyprland-git hyprland
```

## Options

| Option | Description |
|--------|-------------|
| `--dry-run` | Preview changes without applying |
| `--backend <BACKEND>` | Specify backend (aur, flatpak, etc.) |
| `-f, --force` | Force replacement |

## Tips

1. **Always dry-run first:**
   ```bash
   declarch switch old new --dry-run
   ```

2. **For multiple changes, edit config instead:**
   ```bash
   declarch edit
   # Make all changes
   declarch sync --prune
   ```

## Related

- [`edit`](edit.md) - Manual config editing for complex changes
- [`sync`](sync.md) - Apply configuration changes
