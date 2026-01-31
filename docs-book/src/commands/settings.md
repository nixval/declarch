# declarch settings

Manage declarch configuration settings.

## Usage

```bash
declarch settings <COMMAND>
```

## Commands

- `set <key> <value>` - Set a setting value
- `get <key>` - Get a setting value
- `show` - Show all settings
- `reset <key>` - Reset setting to default

## Settings

### Display Settings

| Setting | Values | Default | Description |
|---------|--------|---------|-------------|
| `color` | auto, always, never | auto | Control colored output |
| `format` | table, json, yaml | table | Output format |
| `verbose` | true, false | false | Verbose logging |
| `compact` | true, false | false | Compact output mode |
| `progress` | on, off | off | Show progress bars |
| `editor` | <command> | (system) | Text editor for edit command |

### Backend Settings

| Setting | Values | Default | Description |
|---------|--------|---------|-------------|
| `backends` | comma-separated list | distro-aware | Enabled package backends |
| `backend_mode` | auto, enabled-only | auto | Backend selection mode |

## Backend Settings Behavior

### Settings vs CLI Flags

**Settings** provide persistent defaults for your workflow:
```bash
# Set once, applies to all commands
declarch settings set backends "aur,flatpak"
declarch settings set backend_mode "enabled-only"
```

**CLI flags** provide temporary overrides when needed:
```bash
# Temporarily search all backends
declarch search rust --backends aur,npm,cargo

# Temporarily use different backends
declarch sync --backends flatpak
```

This pattern is intentional and matches common CLI tools (git, npm, etc.).

### backend_mode Values

**auto** (default):
- Automatically detects available backends
- Uses all backends that have search/sync support
- Distro-aware (excludes AUR on non-Arch systems)

**enabled-only**:
- Only uses backends listed in `backends` setting
- For precise control over which backends are used
- Useful for systems with some backends unavailable

### backends Setting

Comma-separated list of enabled backends:
```bash
# Enable specific backends
declarch settings set backends "aur,flatpak,npm"

# Check current setting
declarch settings get backends

# Reset to distro-aware defaults
declarch settings reset backends
```

**Available backends:** `aur`, `flatpak`, `soar`, `npm`, `yarn`, `pnpm`, `bun`, `pip`, `cargo`, `brew`

### Distro-Aware Defaults

On **Arch-based systems** (Arch, Manjaro, EndeavourOS):
```
backends: aur,flatpak,soar,npm,cargo,pip,brew
```

On **non-Arch systems** (Debian, Fedora, etc.):
```
backends: flatpak,soar,npm,cargo,pip,brew
```

AUR is automatically excluded on non-Arch systems.

## Examples

### View All Settings
```bash
declarch settings show
```

### Set Color Mode
```bash
# Disable colors
declarch settings set color never

# Re-enable colors
declarch settings set color auto
```

### Configure Backends
```bash
# Only use specific backends
declarch settings set backend_mode "enabled-only"
declarch settings set backends "aur,flatpak"

# Now search/sync only uses AUR and Flatpak
declarch search rust
declarch sync
```

### Temporary Override
```bash
# Even with backends="aur,flatpak", temporarily search npm too
declarch search rust --backends aur,flatpak,npm
```

### Reset to Defaults
```bash
declarch settings reset backends
declarch settings reset backend_mode
```

## Related

- [`search`](search.md) - Search respects backend settings
- [`sync`](sync.md) - Sync respects backend settings
- [`init`](init.md) - Sets distro-aware defaults on first run
