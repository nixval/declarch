# declarch info

Show system status and managed packages.

## Usage

```bash
declarch info
```

## Description

The `info` command displays comprehensive information about:

- Configuration file location and validity
- State file location and statistics
- Managed packages per backend
- System package counts

## Examples

### Basic Info

```bash
declarch info
```

Output:
```
=== Declarch System Info ===

Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Configured Packages: 15
  AUR: 5 packages
  Flatpak: 3 packages
  npm: 4 packages
  Cargo: 3 packages

Managed Packages: 12
  AUR: 4 packages
  Flatpak: 3 packages
  npm: 3 packages
  Cargo: 2 packages

Unadopted Packages: 3
  AUR: hyprland, waybar
  npm: typescript
```

## Understanding the Output

### Configuration Section

```
Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json
```

Shows file locations. If files are missing, will show:
```
Configuration: NOT FOUND
State: NOT FOUND
```

### Configured Packages

Packages declared in your configuration files:

```
Configured Packages: 15
  AUR: 5 packages
  Flatpak: 3 packages
  npm: 4 packages
  Cargo: 3 packages
```

This includes:
- Packages in main `declarch.kdl`
- Packages in imported modules
- All merged configurations

### Managed Packages

Packages that declarch is actively tracking:

```
Managed Packages: 12
  AUR: 4 packages
  Flatpak: 3 packages
  npm: 3 packages
  Cargo: 2 packages
```

These are packages that:
1. Are in your configuration
2. Have been synced (installed or adopted) at least once
3. Exist in the state file

### Unadopted Packages

Packages in config but not yet in state:

```
Unadopted Packages: 3
  AUR: hyprland, waybar
  npm: typescript
```

These packages:
- Are declared in your config
- But haven't been synced yet
- Will be installed on next `declarch sync`

## Use Cases

### Check System Status

Quick overview of what's managed:
```bash
declarch info
```

### Verify Sync Worked

After running sync, check if unadopted count decreased:
```bash
# Before sync
declarch info
# Unadopted Packages: 3

declarch sync

# After sync
declarch info
# Unadopted Packages: 0
```

### Debug Missing Packages

If a package isn't being managed:
```bash
declarch info
# Unadopted Packages: typescript

declarch check --verbose
# Look for typescript in output
```

### Compare Config vs State

See differences between what you want and what you have:
```bash
declarch info

# Configured: 15
# Managed: 12
# Gap: 3 packages need sync
```

## Detailed Output Examples

### Fresh Installation

```
=== Declarch System Info ===

Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Configured Packages: 10
  AUR: 5 packages
  Flatpak: 2 packages
  npm: 3 packages

Managed Packages: 0
No packages are being tracked yet.

Unadopted Packages: 10
  AUR: bat, exa, ripgrep, fd, fzf
  Flatpak: com.spotify.Client, org.mozilla.firefox
  npm: typescript, prettier, eslint

Run 'declarch sync' to install and adopt these packages.
```

### After Sync

```
=== Declarch System Info ===

Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Configured Packages: 10
  AUR: 5 packages
  Flatpak: 2 packages
  npm: 3 packages

Managed Packages: 10
  AUR: bat, exa, ripgrep, fd, fzf (5)
  Flatpak: com.spotify.Client, org.mozilla.firefox (2)
  npm: typescript, prettier, eslint (3)

Unadopted Packages: 0
All configured packages are being managed.
```

### With Orphaned Packages

```
=== Declarch System Info ===

Configuration: ~/.config/declarch/declarch.kdl
State: ~/.local/state/declarch/state.json

Configured Packages: 5
  AUR: 3 packages

Managed Packages: 8
  AUR: bat, exa, ripgrep (3)
  Flatpak: com.spotify.Client, org.mozilla.firefox (2)
  npm: typescript, prettier, eslint (3)

Orphaned Packages: 3
  Flatpak: com.spotify.Client, org.mozilla.firefox (removed from config)
  npm: typescript, prettier, eslint (removed from config)

These packages are managed but no longer in your configuration.
Run 'declarch sync --prune' to remove them.
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Info displayed successfully |
| 1 | Error reading configuration or state |

## Integration with Other Commands

### With Check

```bash
declarch check
declarch info
```

Check validates syntax, info shows status.

### With Sync

```bash
declarch info
declarch sync
declarch info
```

Compare before/after states.

### In Scripts

```bash
#!/bin/bash
# Check if packages are unadopted
UNADOPTED=$(declarch info | grep "Unadopted Packages" | awk '{print $3}')

if [ "$UNADOPTED" -gt 0 ]; then
    echo "Unadopted packages found, syncing..."
    declarch sync
fi
```

## Troubleshooting

### "Configuration: NOT FOUND"

**Cause:** Config file doesn't exist

**Solution:**
```bash
declarch init
```

### "State: NOT FOUND"

**Cause:** State file doesn't exist

**Solution:**
```bash
# Will be created on first sync
declarch sync
```

### "Managed Packages: 0" but packages exist

**Cause:** Packages were installed before using declarch

**Solution:**
```bash
# Run sync to adopt existing packages
declarch sync
```

### Unadopted packages persist after sync

**Cause:** Installation failed or package doesn't exist

**Solution:**
```bash
# Check package exists
paru -Si package-name

# Sync with verbose output
declarch sync -v

# Or remove from config
declarch edit
```

## Related Commands

- [`check`](check.md) - Validate configuration
- [`sync`](sync.md) - Install and adopt packages
- [`init`](init.md) - Initialize configuration

## Tips

1. **Check status before and after sync:**
   ```bash
   declarch info | grep "Unadopted"
   declarch sync
   declarch info | grep "Unadopted"
   ```

2. **Monitor package counts:**
   ```bash
   # Watch for package creep
   declarch info | grep "Managed Packages"
   ```

3. **Find orphaned packages:**
   ```bash
   declarch info | grep -A 10 "Orphaned"
   ```

4. **Use in status bar:**
   ```bash
   # Show managed package count in status bar
   declarch info | grep "Managed Packages" | awk '{print $3}'
   ```
