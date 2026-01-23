# declarch switch

Switch between package variants (e.g., stable → git version).

## Usage

```bash
declarch switch <OLD_PACKAGE> <NEW_PACKAGE> [OPTIONS]
```

## Description

The `switch` command replaces one package with another. This is useful for:

- Switching from stable to git versions: `hyprland` → `hyprland-git`
- Testing alternatives: `firefox` → `firefox-beta`
- Replacing deprecated packages

The command:
1. Removes the old package from config
2. Adds the new package to config
3. Installs the new package
4. Removes the old package

## Arguments

| Argument | Description |
|----------|-------------|
| `OLD_PACKAGE` | Package name to remove |
| `NEW_PACKAGE` | Package name to install |

## Options

| Option | Description |
|--------|-------------|
| `--backend <BACKEND>` | Package manager backend (aur, flatpak, soar) |
| `--dry-run` | Preview changes without executing |
| `-f, --force` | Force replacement |

## Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Quiet mode |
| `-y, --yes` | Skip confirmation prompts |

## Examples

### Basic Switch

```bash
declarch switch hyprland hyprland-git
```

This:
1. Removes `hyprland` from config
2. Adds `hyprland-git` to config
3. Installs `hyprland-git`
4. Removes `hyprland`

### Dry Run First

```bash
declarch switch firefox firefox-beta --dry-run
```

Output:
```
[DRY RUN] Would switch:
  - firefox (aur)
  + firefox-beta (aur)

[DRY RUN] Config changes:
  Remove: firefox from packages block
  Add: firefox-beta to packages block

[DRY RUN] Run without --dry-run to apply changes.
```

### Switch Specific Backend

```bash
declarch switch firefox firefox-beta --backend aur
```

Explicitly specifies AUR backend.

### Force Switch

```bash
declarch switch pipewire pipewire-full -f
```

Forces the replacement without additional prompts.

### Verbose Output

```bash
declarch switch vim neovim -v
```

Shows detailed operation information.

## How Switch Works

### Phase 1: Config Update

1. Reads current configuration
2. Finds OLD_PACKAGE in config
3. Removes OLD_PACKAGE
4. Adds NEW_PACKAGE in its place
5. Writes updated config

### Phase 2: Package Operations

1. Installs NEW_PACKAGE
2. Removes OLD_PACKAGE
3. Updates state file

### Example Transformation

**Before:**
```kdl
packages {
    hyprland
    waybar
}
```

**Command:**
```bash
declarch switch hyprland hyprland-git
```

**After:**
```kdl
packages {
    hyprland-git
    waybar
}
```

## Switching Across Backends

### AUR to Flatpak

```bash
declarch switch firefox com.mozilla.firefox --backend flatpak
```

Wait, `switch` only works within the same backend. To switch between backends:

1. Manually edit config:
```kdl
packages {
    // firefox removed
}

packages:flatpak {
    com.mozilla.firefox  // added
}
```

2. Then sync:
```bash
declarch sync --prune
```

### With Backend Prefix in Package Name

You can specify backend in the package name:

```bash
# AUR
declarch switch firefox firefox-beta

# Flatpak (note: this doesn't work as expected)
# Use manual config edit instead
```

For cross-backend switches, edit config manually.

## Output Examples

### Successful Switch

```bash
$ declarch switch hyprland hyprland-git

=== Configuration Update ===
✓ Removed hyprland from config
✓ Added hyprland-git to config

=== Package Operations ===
✓ Installing hyprland-git...
✓ Removing hyprland...

=== Summary ===
Switched: hyprland → hyprland-git
```

### Dry Run Output

```bash
$ declarch switch hyprland hyprland-git --dry-run

[DRY RUN] Would switch:
  - hyprland (aur)
  + hyprland-git (aur)

[DRY RUN] Config file: ~/.config/declarch/declarch.kdl
[DRY RUN] Run without --dry-run to apply changes.
```

### Package Not Found

```bash
$ declarch switch nonexistent new-package

✗ Package 'nonexistent' not found in configuration

Available packages:
  - hyprland
  - waybar
  - wofi
```

## Use Cases

### Stable to Git Version

```bash
declarch switch hyprland hyprland-git
declarch switch waybar waybar-git
```

### Testing Beta Versions

```bash
declarch switch firefox firefox-beta
declarch switch neovim neovim-nightly
```

### Replacing Deprecated Packages

```bash
declarch switch exa eza
declarch switch bat bat-extras
```

### A/B Testing

```bash
# Try alternative
declarch switch pipewire pulseaudio

# Switch back if needed
declarch switch pulseaudio pipewire
```

## Common Workflows

### Test Git Version

```bash
# Switch to git version
declarch switch hyprland hyprland-git

# Test it out
# ... use system for a while ...

# Switch back if problems
declarch switch hyprland-git hyprland
```

### Gradual Migration

```bash
# Check what would change
declarch switch package package-new --dry-run

# If satisfied, apply
declarch switch package package-new
```

### Batch Switching

Switch multiple packages:

```bash
declarch switch hyprland hyprland-git
declarch switch waybar waybar-git
declarch switch wofi wofi-git
```

Or edit config manually for multiple changes:
```kdl
packages {
    hyprland-git
    waybar-git
    wofi-git
}
```

## Safety Considerations

### Always Dry Run First

```bash
declarch switch old new --dry-run
declarch switch old new
```

### Check Dependencies

Some packages are dependencies of others. Switching may:

1. Break dependent packages
2. Trigger large dependency tree changes

**Before switching:**
```bash
# Check what depends on package
pactree -r hyprland

# Check reverse dependencies
pactree hyprland-git
```

### Backup Configuration

Declarch automatically backs up config before switching:
```bash
~/.config/declarch/declarch.kdl.backup
```

### Protected Packages

Switching won't remove protected packages:

```kdl
policy {
    protected {
        linux
        systemd
    }
}
```

```bash
declarch switch linux linux-lts
# ✗ Cannot switch protected package
```

## Troubleshooting

### "Package not found in configuration"

**Cause:** OLD_PACKAGE not in your config

**Solution:**
```bash
# Check what's in config
declarch check --verbose

# Add package first, then switch
declarch edit
```

### "Backend not available"

**Cause:** Required package manager not installed

**Solution:**
```bash
# Install package manager
paru -S paru  # or yay, flatpak, etc.
```

### "Dependencies conflict"

**Cause:** NEW_PACKAGE has conflicting dependencies

**Solution:**
```bash
# Check dependencies manually
paru -Si new-package

# Or use force
declarch switch old new -f
```

## Related Commands

- [`edit`](edit.md) - Manual config editing for complex changes
- [`sync`](sync.md) - Apply configuration changes
- [`check`](check.md) - Validate configuration

## Tips

1. **Always dry run first:**
   ```bash
   declarch switch old new --dry-run
   ```

2. **Check package info before switching:**
   ```bash
   paru -Si new-package
   ```

3. **Use verbose mode for debugging:**
   ```bash
   declarch switch old new -v
   ```

4. **For multiple switches, edit config manually:**
   ```bash
   declarch edit
   # Make all changes at once
   declarch sync --prune
   ```

5. **Test git versions before committing:**
   ```bash
   declarch switch pkg pkg-git
   # Test...
   declarch switch pkg-git pkg  # Switch back if needed
   ```
