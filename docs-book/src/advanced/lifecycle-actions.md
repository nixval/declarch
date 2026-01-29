# Lifecycle Actions System

Run commands before and after package synchronization operations.

## What Are Lifecycle Actions?

lifecycle actions are shell commands that run at specific points during the `declarch sync` process. They allow you to:

- Send notifications before/after updates
- Restart services after package installations
- Run system maintenance tasks
- Execute custom scripts based on package changes

## Security Warning

⚠️ **lifecycle actions are disabled by default for security.**

Remote configurations (from `declarch init <url>`) may contain arbitrary commands. Always review lifecycle actions before enabling.

## Action Types (v0.4.4)

### Global Actions

Run at specific sync phases:

- **`pre-sync`** - Before any package operations
- **`post-sync`** - After all package operations
- **`on-success`** - After successful sync
- **`on-failure`** - After failed sync

### Package Actions

Run for specific packages:

- **`pre-install`** - Before installing a package
- **`post-install`** - After installing a package
- **`pre-remove`** - Before removing a package
- **`post-remove`** - After removing a package
- **`on-update`** - When a package is updated

## Syntax

### Global Actions

```kdl
hooks {
    pre-sync "echo 'Starting sync...'"
    post-sync "notify-send 'Done'"
    on-success "notify-send 'Success!'"
    on-failure "notify-send 'Failed!'"
}
```

### Package Actions (Block Syntax)

```kdl
hooks {
    docker {
        post-install "systemctl enable docker" --sudo
        post-remove "systemctl disable docker" --sudo
    }

    nvidia {
        post-install "mkinitcpio -P" --sudo --required
    }
}
```

### Package Actions (Shorthand Syntax)

```kdl
hooks {
    docker:post-install "systemctl enable docker" --sudo
    waybar:post-install "systemctl --user restart waybar"
}
```

### Mixed Syntax

```kdl
hooks {
    // Global hooks
    pre-sync "echo 'Starting...'"

    // Package block (good for multiple hooks)
    docker {
        post-install "systemctl enable docker" --sudo
        post-remove "docker system prune -f"
    }

    // Shorthand (good for single hook)
    waybar:post-install "systemctl --user restart waybar"
}
```

## Flags

### Execution Flags

| Flag | Description |
|------|-------------|
| `--sudo` | Run with sudo privileges |
| `--required` | Fail sync if hook errors |
| `--ignore` | Silently ignore hook errors |

Default behavior: Warn on error (doesn't fail sync)

### Examples

```kdl
hooks {
    // Regular hook (no sudo)
    post-sync "notify-send 'Done'"

    // Run with sudo
    post-sync "systemctl restart gdm" --sudo

    // Critical hook - fail if errors
    nvidia:post-install "mkinitcpio -P" --sudo --required

    // Non-critical - ignore errors
    post-sync "cleanup.sh" --ignore
}
```

## Basic Usage

### Enable Hooks

lifecycle actions are **disabled by default**. Enable with:

```bash
declarch sync --hooks
```

### Add Global Actions

```kdl
// declarch.kdl

hooks {
    pre-sync "notify-send 'Starting sync...'"
    post-sync "notify-send 'Packages updated'"
}
```

### Add Package Actions

```kdl
// declarch.kdl

packages {
    docker
}

hooks {
    docker {
        post-install "systemctl enable --now docker" --sudo
    }
}
```

## Examples

### Docker Service

```kdl
packages {
    docker
}

hooks {
    docker {
        post-install "systemctl enable --now docker" --sudo --required
        post-remove "systemctl disable docker" --sudo
    }
}
```

### NVIDIA Drivers

```kdl
packages {
    nvidia
}

hooks {
    nvidia:post-install "mkinitcpio -P" --sudo --required
}
```

### Desktop Environment

```kdl
packages {
    gdm
    waybar
}

hooks {
    gdm:post-install "systemctl enable gdm" --sudo

    waybar {
        post-install "systemctl --user enable waybar"
        post-install "pkill -SIGUSR1 waybar"
    }
}
```

### Development Tools

```kdl
packages {
    rust
}

hooks {
    rust:post-install "rustup component add rust-analyzer"
}
```

## Error Handling

### Default Behavior (Warn)

```kdl
hooks {
    // Warns if this fails, but doesn't stop sync
    post-sync "cleanup.sh"
}
```

### Required Hooks

```kdl
hooks {
    // Fails sync if this hook errors
    nvidia:post-install "mkinitcpio -P" --sudo --required
}
```

### Ignore Errors

```kdl
hooks {
    // Silently ignores errors
    post-sync "optional-cleanup.sh" --ignore
}
```

## Validation

### No Embedded Sudo

❌ **Don't** embed `sudo` in commands:

```kdl
// ERROR: Use --sudo flag instead
docker:post-install "sudo systemctl enable docker" --sudo
```

✅ **Do** use the `--sudo` flag:

```kdl
docker:post-install "systemctl enable docker" --sudo
```

## Hooks in Modules

Hooks work in modules too:

```kdl
// modules/docker.kdl

packages {
    docker
}

hooks {
    docker {
        post-install "systemctl enable docker" --sudo
    }
}
```

### Hook Merging

When importing modules with hooks, lifecycle actions are accumulated:

**Root config:**
```kdl
hooks {
    post-sync "notify-send 'Root sync complete'"
}
```

**Module (docker.kdl):**
```kdl
hooks {
    docker:post-install "systemctl enable docker" --sudo
}
```

**Result:** Both hooks run.

## Security Best Practices

### 1. Review Remote Configs

Always review before enabling hooks from remote configs:

```bash
declarch init myuser/dotfiles
cat ~/.config/declarch/declarch.kdl | grep -A 10 "hooks"
```

### 2. Use --dry-run First

```bash
declarch sync --dry-run --hooks
```

### 3. Prefer User-Level Hooks

```kdl
// ✅ Prefer user-level
waybar:post-install "systemctl --user restart waybar"

// ❌ Avoid sudo unless needed
gdm:post-install "systemctl restart gdm" --sudo
```

### 4. Use --required for Critical Hooks

```kdl
// Critical system update - must succeed
nvidia:post-install "mkinitcpio -P" --sudo --required
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

### Sudo Password Prompt

**Cause:** Hook uses `--sudo` flag

**Solution:**
Configure sudoers to allow specific commands without password:
```
username ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart gdm
```

## Hook Ordering

Hooks run in this order:

1. **Pre-sync** hooks (global, in order defined)
2. **Pre-install** hooks (per package, in order defined)
3. Package operations (install, remove, update)
4. **Post-install** hooks (per package, in order defined)
5. **Post-sync** hooks (global, in order defined)
6. **On-success** or **On-failure** hooks

Example:
```kdl
hooks {
    pre-sync "echo 'Step 1'"
    pre-sync "echo 'Step 2'"

    docker:post-install "echo 'Step 3'"

    post-sync "echo 'Step 4'"
    post-sync "echo 'Step 5'"
}
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

### Service Management

```kdl
hooks {
    docker:post-install "systemctl enable --now docker" --sudo --required
    docker:post-remove "docker system prune -f"
}
```

### Notification Chain

```kdl
hooks {
    post-sync "notify-send 'Packages updated'"
    waybar:post-install "pkill -SIGUSR1 waybar"
}
```

### Conditional Logic in Scripts

```kdl
hooks {
    post-sync "~/.config/declarch/scripts/post-sync.sh"
}
```

```bash
#!/bin/bash
# ~/.config/declarch/scripts/post-sync.sh

# Only run on desktop
if hostname | grep -q "desktop"; then
    systemctl restart gdm
fi

# Send notification
notify-send "Declarch" "Sync completed"
```

## Related

- [KDL Syntax Reference](../configuration/kdl-syntax.md) - Configuration syntax
- [Modules Guide](../configuration/modules.md) - Hooks in modules
- [Sync Command](../commands/sync.md) - Enabling hooks with --hooks flag

## Migration from v0.4.3

### Old Syntax (Deprecated)

```kdl
// Old (v0.4.3)
on-sync "notify-send 'Done'"
on-sync-sudo "systemctl restart gdm"
hooks {
    post-sync {
        run "notify-send 'Done'"
        sudo-needed "systemctl restart gdm"
    }
}
```

### New Syntax (v0.4.4)

```kdl
// New (v0.4.4)
hooks {
    post-sync "notify-send 'Done'"
    post-sync "systemctl restart gdm" --sudo
}
```

### Key Changes

1. **`on-sync-sudo`** → **`--sudo` flag**
2. **`run`** / **`sudo-needed`** → removed (use flags)
3. **`on-pre-sync`** → **`pre-sync`** (in hooks block)
4. **`HookType` enum** simplified to `User` / `Root`
5. **Per-package hooks** added

## Tips

1. **Keep hooks simple:** One command per hook
2. **Use scripts for complex logic:** Put multiple commands in a script file
3. **Test hooks manually:** Run hook commands directly before adding to config
4. **Use notifications:** Get feedback on hook execution
5. **Avoid long-running hooks:** Hooks block sync completion
