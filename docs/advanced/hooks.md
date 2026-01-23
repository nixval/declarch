# Hooks System

Run commands before and after package synchronization operations.

## What Are Hooks?

Hooks are shell commands that run at specific points during the `declarch sync` process. They allow you to:

- Send notifications before/after updates
- Restart services after package installations
- Run system maintenance tasks
- Execute custom scripts based on package changes

## Security Warning

⚠️ **Hooks are disabled by default for security.**

Remote configurations (from `declarch init <url>`) may contain arbitrary commands. Always review hooks before enabling.

## Hook Types

### Pre-Sync Hook

Runs **before** any package operations.

```kdl
on-pre-sync "notify-send 'Starting package sync...'"
```

Uses:
- Notify user of pending updates
- Create system backups
- Prepare system for changes

### Post-Sync Hook

Runs **after** successful package operations (without sudo).

```kdl
on-sync "notify-send 'Packages updated successfully'"
on-sync "pkill -SIGUSR1 dunst"  // Refresh notification daemon
```

Uses:
- Send success notifications
- Refresh application caches
- Update system status

### Post-Sync Sudo Hook

Runs **after** successful package operations (with sudo privileges).

```kdl
on-sync-sudo "systemctl restart gdm"
on-sync-sudo "ldconfig"
```

Uses:
- Restart services
- Update system cache
- Modify system configuration

## Basic Usage

### Enable Hooks

Hooks are **disabled by default**. Enable with:

```bash
declarch sync --hooks
```

### Add Hooks to Config

```kdl
// declarch.kdl

on-pre-sync "notify-send 'Starting sync...'"
on-sync "notify-send 'Packages updated'"
on-sync-sudo "systemctl restart gdm"
```

### Run with Hooks

```bash
declarch sync --hooks
```

## Hook Syntax

### Flat Syntax (Recommended)

```kdl
on-pre-sync "command1"
on-sync "command2"
on-sync-sudo "command3"
```

### Nested Syntax (Still Supported)

```kdl
hooks {
    pre-sync {
        run "notify-send 'Starting sync...'"
    }
    post-sync {
        run "notify-send 'Packages updated'"
        sudo-needed "systemctl restart gdm"
    }
}
```

## Examples

### Desktop Notifications

```kdl
on-pre-sync "notify-send -t 5000 'Declarch' 'Starting package synchronization...'"
on-sync "notify-send -t 5000 'Declarch' 'Packages updated successfully!'"
```

### Restart Display Server

```kdl
on-sync-sudo "systemctl restart gdm"
```

### Update Application Databases

```kdl
on-sync "update-desktop-database ~/.local/share/applications"
on-sync-sudo "ldconfig"
```

### Refresh Waybar

```kdl
on-sync "pkill -SIGUSR1 waybar"
```

### Generate Fonts Cache

```kdl
on-sync-sudo "fc-cache -fv"
```

### Clean Up

```kdl
on-sync "journalctl --vacuum-time=7d"
on-sync "rm -rf ~/.cache/pacman/pkg"
```

### Backup System

```kdl
on-pre-sync "timeshift --create --comments 'Before declarch sync'"
```

### Custom Scripts

```kdl
on-pre-sync "~/.config/declarch/scripts/pre-sync.sh"
on-sync "~/.config/declarch/scripts/post-sync.sh"
```

## Hook Scripts

### Create Hook Script

```bash
# ~/.config/declarch/scripts/post-sync.sh

#!/bin/bash

# Refresh font cache
fc-cache -fv &> /dev/null &

# Update GTK icon cache
update-desktop-database ~/.local/share/applications &> /dev/null &

# Send notification
notify-send "Declarch" "Sync completed"

# Restart Waybar if running
if pgrep -x waybar > /dev/null; then
    pkill -SIGUSR1 waybar
fi
```

Make it executable:
```bash
chmod +x ~/.config/declarch/scripts/post-sync.sh
```

Reference in config:
```kdl
on-sync "~/.config/declarch/scripts/post-sync.sh"
```

## Conditional Hooks

### Run Only If Specific Packages Changed

```bash
#!/bin/bash
# post-sync.sh

# Check if hyprland was installed
if declarch info | grep -q "hyprland"; then
    echo "Hyprland installed, running setup..."
    # Run hyprland setup
fi
```

### Run Only on Success

Hooks only run on successful sync. If sync fails, hooks don't run.

### Environment-Aware Hooks

```bash
#!/bin/bash
# post-sync.sh

# Only run on desktop
if hostname | grep -q "desktop"; then
    systemctl restart gdm
fi

# Only run if gaming packages installed
if declarch check --verbose | grep -q "steam"; then
    notify-send "Gaming packages updated"
fi
```

## Hooks in Modules

Hooks work in modules too:

```kdl
// modules/desktop/hyprland.kdl

on-sync "notify-send 'Hyprland packages updated'"
on-sync-sudo "systemctl restart --user hyprland"
```

### Hook Merging

When importing modules with hooks, hooks are accumulated:

**Root config:**
```kdl
on-sync "notify-send 'Sync complete'"
```

**Module (desktop/hyprland.kdl):**
```kdl
on-sync "notify-send 'Hyprland updated'"
```

**Result:** Both hooks run.

## Security Best Practices

### 1. Review Remote Configs

Always review before enabling hooks from remote configs:

```bash
declarch init myuser/dotfiles
cat ~/.config/declarch/declarch.kdl | grep -A 10 "on-"
```

### 2. Use --dry-run First

```bash
declarch sync --dry-run --hooks
```

This shows which hooks would run (in future versions).

### 3. Avoid Destructive Commands

Don't use hooks like:
```kdl
// ❌ Dangerous
on-sync "rm -rf ~/important-data"
```

### 4. Use Non-Sudo When Possible

Prefer `on-sync` over `on-sync-sudo`:

```kdl
// ✅ Prefer user-level
on-sync "pkill waybar"

// ❌ Avoid sudo unless needed
on-sync-sudo "systemctl restart gdm"
```

### 5. Quote Commands Properly

```kdl
// ✅ Good
on-sync "notify-send 'Message here'"

// ❌ Bad - quoting issues
on-sync "notify-send 'Message with ' quotes'"
```

## Troubleshooting

### Hooks Not Running

**Cause:** Hooks disabled

**Solution:**
```bash
declarch sync --hooks
```

### Hook Command Fails

**Cause:** Command not found or invalid

**Solution:**
```bash
# Test command manually
notify-send 'Test'

# Check hook syntax
declarch check
```

### Permission Denied

**Cause:** Script not executable

**Solution:**
```bash
chmod +x ~/.config/declarch/scripts/post-sync.sh
```

### Sudo Password Prompt

**Cause:** `on-sync-sudo` requires password

**Solution:**
Configure sudoers to allow specific commands without password:
```
username ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart gdm
```

## Hook Ordering

Hooks run in this order:

1. Pre-sync hooks (in order defined)
2. Package operations (install, remove, adopt)
3. Post-sync hooks (in order defined)
4. Post-sync sudo hooks (in order defined)

Example:
```kdl
on-pre-sync "echo 'Step 1'"
on-pre-sync "echo 'Step 2'"

on-sync "echo 'Step 3'"
on-sync "echo 'Step 4'"

on-sync-sudo "echo 'Step 5'"
```

Output:
```
Step 1
Step 2
[package operations...]
Step 3
Step 4
Step 5
```

## Advanced Patterns

### Notify Only on Changes

```bash
#!/bin/bash
# post-sync.sh

# Check if any packages were installed/removed
if declarch sync --dry-run 2>/dev/null | grep -E "^\+|^-" > /dev/null; then
    notify-send "Declarch" "Packages were updated"
fi
```

### Service Management

```kdl
// Restart service only if its packages were updated
on-sync-sudo "systemctl try-restart gdm || true"
```

### Conditional Hooks Based on Exit Code

```kdl
on-sync "command || notify-send 'Command failed'"
```

### Chain Multiple Commands

```kdl
on-sync "command1 && command2 && command3"
```

Or use a script:
```kdl
on-sync "~/.config/declarch/scripts/post-sync.sh"
```

## Related

- [KDL Syntax Reference](../configuration/kdl-syntax.md) - Configuration syntax
- [Modules Guide](../configuration/modules.md) - Hooks in modules
- [Sync Command](../commands/sync.md) - Enabling hooks with --hooks flag

## Tips

1. **Keep hooks simple:** One command per hook
2. **Use scripts for complex logic:** Put multiple commands in a script file
3. **Test hooks manually:** Run hook commands directly before adding to config
4. **Use notifications:** Get feedback on hook execution
5. **Avoid long-running hooks:** Hooks block sync completion

## Examples Gallery

### Desktop Setup

```kdl
on-pre-sync "notify-send 'Starting system update...'"
on-sync "update-desktop-database ~/.local/share/applications"
on-sync "fc-cache -fv"
on-sync-sudo "systemctl restart gdm"
```

### Development Setup

```kdl
on-sync "cargo install-update -a"
on-sync "npm update -g"
on-sync "pip list --outdated"
```

### Gaming Setup

```kdl
on-sync "notify-send 'Steam packages updated'"
on-sync-sudo "systemctl restart steam"
```

### Minimal Setup

```kdl
on-sync "notify-send 'Sync complete'"
```
