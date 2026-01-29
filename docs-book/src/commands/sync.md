# declarch sync

Synchronize system packages with your declarch configuration.

## Usage

```bash
declarch sync [OPTIONS]
declarch sync [FLAGS]
```

## Description

The `sync` command is the primary command in declarch. It:

1. Reads your configuration file
2. Checks current system state
3. Installs missing packages
4. Adopts already-installed packages
5. Optionally removes unwanted packages (with `--prune`)

## Options

| Option | Description |
|--------|-------------|
| `-u, --update` | Update system (runs `paru -Syu` before syncing) |
| `--dry-run` | Preview changes without executing |
| `--prune` | Remove managed packages not in configuration |
| `--gc` | Garbage collect system orphans after sync |
| `--target <TARGET>` | Sync only specific backend or package |
| `--noconfirm` | Skip package manager confirmation prompts |
| `--hooks` | Enable hooks (disabled by default for security) |
| `--skip-soar-install` | Skip automatic Soar installation |

## Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Quiet mode |
| `-y, --yes` | Skip confirmation prompts |
| `-f, --force` | Force operations |

## Examples

### Basic Sync

```bash
declarch sync
```

Installs missing packages and adopts existing ones.

### Sync with System Update

```bash
declarch sync -u
```

Equivalent to:
```bash
paru -Syu  # Update system
declarch sync  # Sync packages
```

### Preview Changes

```bash
declarch sync --dry-run
```

Output example:
```
[DRY RUN] Would install:
  + bat (aur)
  + exa (aur)
  + com.spotify.Client (flatpak)

[DRY RUN] Would remove: (none)
```

### Remove Unwanted Packages

```bash
# First, see what would be removed
declarch sync --prune --dry-run

# If satisfied, remove them
declarch sync --prune
```

### Target Specific Backend

```bash
# Only sync AUR packages
declarch sync --target aur

# Only sync Flatpak packages
declarch sync --target flatpak

# Only sync npm packages
declarch sync --target npm
```

### Target Specific Package

```bash
declarch sync --target firefox
```

Only processes the `firefox` package.

### Garbage Collection

```bash
declarch sync --gc
```

Removes orphan packages after syncing.

### CI/CD Usage

```bash
declarch sync --noconfirm --hooks
```

Skips all prompts and enables hooks for automated environments.

## How Sync Works

### Phase 1: Configuration Parsing

1. Reads `~/.config/declarch/declarch.kdl`
2. Resolves all imports (modules)
3. Merges all configurations
4. Validates syntax

### Phase 2: State Comparison

For each package in config:

| Config | System | Action |
|--------|--------|--------|
| ✅ | ❌ | Install |
| ✅ | ✅ | Adopt (track) |
| ❌ | ✅ (managed) | Keep (unless `--prune`) |
| ❌ | ✅ (unmanaged) | Ignore |

### Phase 3: Package Operations

**Installation:**
```bash
# AUR packages
paru -S --noconfirm bat exa ripgrep

# Flatpak
flatpak install -y com.spotify.Client

# npm
npm install -g typescript prettier
```

**Adoption:**
Simply adds to state without reinstalling:
```json
{
  "packages": {
    "aur": ["bat", "exa"]
  }
}
```

**Pruning (with `--prune`):**
```bash
# Remove packages that are in state but not in config
paru -Rns old-package
flatpak uninstall old-app
```

## Sync Modes

### Default Mode (Safe)

Only **adds** packages, never removes:

```bash
declarch sync
```

Behavior:
- ✅ Install missing
- ✅ Adopt existing
- ❌ Never remove

### Prune Mode (Strict)

Removes managed packages not in config:

```bash
declarch sync --prune
```

Behavior:
- ✅ Install missing
- ✅ Adopt existing
- ✅ Remove unwanted (managed only)

### Dry Run Mode (Preview)

Shows what would happen:

```bash
declarch sync --dry-run
```

**Always use dry-run before prune!**

## Output Examples

### Successful Sync

```bash
$ declarch sync

=== Checking configuration ===
✓ Configuration valid

=== Synchronizing packages ===
✓ bat (aur) - already installed, adopted
✓ exa (aur) - installed
✓ ripgrep (aur) - already installed, adopted
+ com.spotify.Client (flatpak) - installed

=== Summary ===
Installed: 2
Adopted: 2
Removed: 0
Errors: 0
```

### Dry Run Output

```bash
$ declarch sync --dry-run

[DRY RUN] Would install:
  + bat (aur)
  + exa (aur)
  + ripgrep (aur)
  + com.spotify.Client (flatpak)

[DRY RUN] Would remove:
  - old-package (aur)

[DRY RUN] Run without --dry-run to apply changes.
```

### With Prune

```bash
$ declarch sync --prune

=== Checking configuration ===
✓ Configuration valid

=== Synchronizing packages ===
✓ bat (aur) - kept
✓ exa (aur) - kept
- old-package (aur) - removed (not in config)

=== Summary ===
Installed: 0
Adopted: 2
Removed: 1
Errors: 0
```

## Common Workflows

### Daily Update

```bash
declarch sync -u
```

### New Machine Setup

```bash
declarch init myuser/dotfiles
declarch sync --dry-run
declarch sync
```

### Cleanup

```bash
declarch sync --prune --dry-run
declarch sync --prune
```

### Specific Backend Sync

```bash
# Only update development tools
declarch sync --target npm --target cargo
```

### Quick Preview

```bash
declarch sync --dry-run | grep "^\+"
# Shows only packages to be installed
```

## Safety Features

### Automatic State Backup

Before modifying state, declarch creates a backup:

```bash
~/.local/state/declarch/state.json.backup
```

### Protected Packages

Never removes critical system packages:

```kdl
policy {
    protected {
        linux
        systemd
        base-devel
    }
}
```

### Confirmation Prompts

Package managers may prompt for confirmation:
```
:: Proceed with installation? [Y/n]
```

Skip with `--noconfirm` or `-y`:
```bash
declarch sync -y
```

### Hooks Disabled by Default

Hooks can run arbitrary commands. Must explicitly enable:

```bash
declarch sync --hooks
```

## Troubleshooting

### "Failed to install package"

**Cause:** Package doesn't exist or network error

**Solution:**
```bash
# Check if package exists
paru -Si package-name

# Check network
ping -c 3 aur.archlinux.org

# Sync specific backend
declarch sync --target aur
```

### "State file corrupted"

**Cause:** Interrupted sync or corrupted state file

**Solution:**
```bash
# Restore from backup
cp ~/.local/state/declarch/state.json.backup ~/.local/state/declarch/state.json

# Or start fresh
rm ~/.local/state/declarch/state.json
declarch sync
```

### Hooks not running

**Cause:** Hooks are disabled by default

**Solution:**
```bash
declarch sync --hooks
```

## Internal Architecture (v0.4.4+)

The `sync` command has been refactored into modular, testable components:

### Main Execution Flow

The `run()` function orchestrates these steps:

1. **Target Resolution** - Parse and validate sync target
2. **Config Loading** - Load root config with optional modules
3. **Pre-sync Hooks** - Execute lifecycle actions before sync
4. **System Update** - Run system package manager update
5. **Manager Initialization** - Initialize available package managers
6. **State Loading** - Load previous sync state
7. **Transaction Resolution** - Determine packages to install/remove/update
8. **Variant Checking** - Detect package variant transitions
9. **Partial Upgrade Warning** - Warn if system not recently updated
10. **Display Plan** - Show what will be done
11. **Execution** - Install/remove packages
12. **State Update** - Save new state

### Helper Functions

- `resolve_and_filter_packages()` - Filter by available backends and resolve
- `check_variant_transitions()` - Detect AUR variant mismatches (e.g., hyprland → hyprland-git)
- `warn_partial_upgrade()` - Display partial upgrade risk warning
- `execute_installations()` - Install packages grouped by backend
- `execute_pruning()` - Remove packages with safety checks
- `initialize_managers_and_snapshot()` - Create package managers and scan system
- `display_transaction_plan()` - Show installation/removal plan
- `update_state_after_sync()` - Save new package state

This modular design makes the code:
- **Easier to test** - Each function has a single responsibility
- **Easier to maintain** - Changes are localized to specific functions
- **More readable** - The main flow is clear and concise
- **More secure** - Security checks are isolated and verifiable

## Related Commands

- [`check`](check.md) - Validate configuration before syncing
- [`info`](info.md) - View system status
- [`init`](init.md) - Initialize or fetch configuration

## Tips

1. **Always dry-run before prune:**
   ```bash
   declarch sync --prune --dry-run
   ```

2. **Use targeting for faster syncs:**
   ```bash
   declarch sync --target flatpak  # Only Flatpak
   ```

3. **Combine with system update:**
   ```bash
   declarch sync -u  # Update + sync
   ```

4. **Verbose mode for debugging:**
   ```bash
   declarch sync -v
   ```

5. **Quiet mode for scripts:**
   ```bash
   declarch sync -q
   ```
