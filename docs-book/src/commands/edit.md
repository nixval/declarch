# declarch edit

Edit declarch configuration files in your preferred editor.

## Usage

```bash
declarch edit [TARGET]
```

## Description

The `edit` command opens configuration files in your editor. It:

1. Determines which editor to use
2. Opens the specified file (or main config by default)
3. Handles modules and subdirectories

## Arguments

### TARGET (Optional)

File or module to edit.

| TARGET | File Opened |
|--------|-------------|
| (none) | `~/.config/declarch/declarch.kdl` |
| `base` | `~/.config/declarch/modules/base.kdl` |
| `desktop` | `~/.config/declarch/modules/desktop.kdl` |
| `dev/python` | `~/.config/declarch/modules/dev/python.kdl` |
| `hyprland/niri-nico` | `~/.config/declarch/modules/hyprland/niri-nico.kdl` |

## Editor Selection

The editor is chosen in this order (first found wins):

1. `editor "name"` in your `declarch.kdl`
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default fallback)

## Examples

### Edit Main Config

```bash
declarch edit
```

Opens `~/.config/declarch/declarch.kdl`

### Edit Module

```bash
declarch edit base
```

Opens `~/.config/declarch/modules/base.kdl`

### Edit Nested Module

```bash
declarch edit dev/rust
declarch edit hyprland/niri-nico
```

Opens:
- `~/.config/declarch/modules/dev/rust.kdl`
- `~/.config/declarch/modules/hyprland/niri-nico.kdl`

### Set Editor in Config

```kdl
// declarch.kdl
editor "nvim"
```

Now `declarch edit` uses neovim.

### Set Editor via Environment

```bash
export EDITOR="code"
export VISUAL="code"

declarch edit
# Opens in VS Code
```

### Use Specific Editor Once

```bash
EDITOR=vim declarch edit
```

Uses vim for this command only.

## How It Works

### Editor Resolution

```
1. Check config for "editor" directive
   └─ Found: Use that editor
   └─ Not found: Continue

2. Check $EDITOR environment variable
   └─ Set: Use that editor
   └─ Not set: Continue

3. Check $VISUAL environment variable
   └─ Set: Use that editor
   └─ Not set: Continue

4. Use default: nano
```

### File Resolution

For `declarch edit <target>`:

```
target → ~/.config/declarch/modules/<target>.kdl
target/sub → ~/.config/declarch/modules/<target>/<sub>.kdl
```

Special cases:
- `(none)` → `~/.config/declarch/declarch.kdl`

## Common Editor Configurations

### Neovim

```kdl
editor "nvim"
```

Or:
```bash
export EDITOR=nvim
```

### VS Code

```kdl
editor "code"
```

Or:
```bash
export EDITOR="code --wait"
```

### Helix

```kdl
editor "hx"
```

### Vim

```kdl
editor "vim"
```

### Nano (Default)

If no editor is set, nano is used automatically.

### Graphical Editors

For GUI editors that don't wait, use `--wait`:

```bash
export EDITOR="code --wait"
export EDITOR="subl --wait"
```

## Module File Organization

### Flat Structure

```
~/.config/declarch/
├── declarch.kdl
└── modules/
    ├── base.kdl
    ├── desktop.kdl
    ├── development.kdl
    └── gaming.kdl
```

Edit:
```bash
declarch edit base
declarch edit desktop
```

### Nested Structure

```
~/.config/declarch/
├── declarch.kdl
└── modules/
    ├── base.kdl
    └── development/
        ├── rust.kdl
        ├── python.kdl
        └── node.kdl
```

Edit:
```bash
declarch edit development/rust
declarch edit development/python
```

### Community Configs Structure

```
~/.config/declarch/
├── declarch.kdl
└── modules/
    └── hyprland/
        ├── niri-nico.kdl
        └── my-setup.kdl
```

Edit:
```bash
declarch edit hyprland/niri-nico
declarch edit hyprland/my-setup
```

## Workflows

### Edit and Check

```bash
declarch edit
declarch check
```

### Edit and Sync

```bash
declarch edit
declarch sync --dry-run
declarch sync
```

### Edit Specific Module

```bash
# Edit module
declarch edit development/rust

# Check it
declarch check

# Sync if valid
declarch sync
```

### Quick Edit with Verification

```bash
declarch edit && declarch check && declarch sync
```

Runs check and sync only if edit exits successfully.

### Edit All Modules

```bash
# Edit each module
declarch edit base
declarch edit desktop
declarch edit development

# Then sync
declarch sync
```

## Integration with Auto-Commands

### Vim/Neovim Auto-Check

Add to `~/.config/nvim/init.lua`:

```lua
-- Auto-run declarch check when saving .kdl files
vim.api.nvim_create_autocmd("BufWritePost", {
  pattern = "*.kdl",
  callback = function()
    vim.cmd("silent !declarch check")
  end,
})
```

### VS Code Tasks

Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "declarch check",
      "type": "shell",
      "command": "declarch check",
      "problemMatcher": []
    },
    {
      "label": "declarch sync dry-run",
      "type": "shell",
      "command": "declarch sync --dry-run",
      "problemMatcher": []
    }
  ]
}
```

## Tips

1. **Set preferred editor in config:**
   ```kdl
   editor "nvim"
   ```

2. **Use environment variable for flexibility:**
   ```bash
   export EDITOR=nvim
   ```

3. **Always check after editing:**
   ```bash
   declarch edit && declarch check
   ```

4. **Use dry-run before syncing:**
   ```bash
   declarch edit
   declarch sync --dry-run
   ```

5. **Chain commands:**
   ```bash
   declarch edit && declarch check && declarch sync
   ```

## Troubleshooting

### Editor doesn't open

**Cause:** Editor not found or not in PATH

**Solution:**
```bash
# Check editor path
which nvim

# Use full path
editor "/usr/bin/nvim"

# Or install editor
paru -S neovim
```

### Graphical editor closes immediately

**Cause:** GUI editor doesn't wait for input

**Solution:**
```bash
# Use --wait flag
export EDITOR="code --wait"
export EDITOR="subl --wait"
```

### Module not found

**Cause:** Module file doesn't exist

**Solution:**
```bash
# Create it
touch ~/.config/declarch/modules/new-module.kdl

# Then edit
declarch edit new-module
```

### Wrong editor selected

**Cause:** Multiple editor settings conflict

**Solution:**
```bash
# Check priority
# 1. Config editor setting
# 2. $EDITOR
# 3. $VISUAL
# 4. nano

# Remove conflicting settings
unset EDITOR
unset VISUAL

# Set in config instead
declarch edit
# Add: editor "nvim"
```

## Related Commands

- [`check`](check.md) - Validate after editing
- [`sync`](sync.md) - Apply changes
- [`init`](init.md) - Create initial config

## See Also

- [KDL Syntax Guide](../configuration/kdl-syntax.md)
- [Modules Guide](../configuration/modules.md)
