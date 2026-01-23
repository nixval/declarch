# Troubleshooting

Common issues and solutions when using declarch.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Configuration Issues](#configuration-issues)
- [Sync Issues](#sync-issues)
- [Package Issues](#package-issues)
- [State Issues](#state-issues)
- [Backend Issues](#backend-issues)
- [Performance Issues](#performance-issues)

## Installation Issues

### "command not found: declarch"

**Symptoms:**
```bash
$ declarch
bash: declarch: command not found
```

**Causes:**
1. Binary not installed
2. Not in PATH

**Solutions:**

Check if installed:
```bash
which declarch
# or
ls /usr/local/bin/declarch
```

If not found, install:
```bash
# From AUR
paru -S declarch

# From source
cargo install declarch
```

Check PATH:
```bash
echo $PATH | grep -o "/usr/local/bin"
```

Add to PATH if missing:
```bash
export PATH="/usr/local/bin:$PATH"
# Add to ~/.bashrc or ~/.zshrc
```

### Permission Denied

**Symptoms:**
```bash
$ declarch sync
bash: /usr/local/bin/declarch: Permission denied
```

**Cause:** Binary not executable

**Solution:**
```bash
sudo chmod +x /usr/local/bin/declarch
```

## Configuration Issues

### "KDL syntax error"

**Symptoms:**
```bash
$ declarch check
✗ KDL syntax error:
   → Line 15: Expected identifier, found '}'

    14 │     packages {
    15 │         bat exa
    16 │     }
```

**Cause:** Invalid KDL syntax

**Solution:**

1. Check the line number
2. Look for:
   - Missing closing braces `}`
   - Invalid characters
   - Incorrect nesting

Common mistakes:
```kdl
// ❌ Wrong: packages on same line without space
packages {bat exa}

// ✅ Correct
packages {
    bat
    exa
}

// ❌ Wrong: missing closing brace
packages {
    bat

// ✅ Correct
packages {
    bat
}
```

### "Failed to resolve import"

**Symptoms:**
```bash
$ declarch check
✗ Failed to resolve import: modules/missing.kdl: No such file or directory
```

**Cause:** Module file doesn't exist

**Solution:**

1. Check if file exists:
```bash
ls ~/.config/declarch/modules/missing.kdl
```

2. Create it:
```bash
touch ~/.config/declarch/modules/missing.kdl
declarch edit missing
```

3. Or remove from imports:
```kdl
imports {
    // modules/missing  // Remove this line
    modules/base
}
```

### Editor Not Opening

**Symptoms:**
```bash
$ declarch edit
# Nothing happens
```

**Cause:** Editor not found or not set correctly

**Solution:**

1. Check editor setting:
```bash
cat ~/.config/declarch/declarch.kdl | grep editor
```

2. Set editor:
```kdl
editor "nvim"
```

3. Or use environment variable:
```bash
export EDITOR=nvim
export VISUAL=nvim
```

## Sync Issues

### "Package not found"

**Symptoms:**
```bash
$ declarch sync
✗ Failed to install nonexistent-pkg: package not found
```

**Cause:** Package doesn't exist in repository

**Solution:**

1. Search for package:
```bash
paru -Ss similar-name
```

2. Check package name spelling
3. Remove from config if doesn't exist

### "Hooks not running"

**Symptoms:**
```bash
$ declarch sync
# Hooks don't execute
```

**Cause:** Hooks disabled by default

**Solution:**
```bash
declarch sync --hooks
```

### Sync Hangs

**Symptoms:**
```bash
$ declarch sync
# Hangs forever
```

**Causes:**
1. Package manager prompt
2. Network issue
3. AUR build taking long

**Solution:**

1. Use verbose mode:
```bash
declarch sync -v
```

2. Use noconfirm:
```bash
declarch sync -y
```

3. Check network:
```bash
ping -c 3 aur.archlinux.org
```

### "--prune removes too much"

**Symptoms:**
```bash
$ declarch sync --prune
# Removes packages you wanted to keep
```

**Cause:** Packages not in config but managed by declarch

**Solution:**

1. Always dry-run first:
```bash
declarch sync --prune --dry-run
```

2. Add wanted packages to config:
```kdl
packages {
    important-pkg
}
```

3. Use `info` to check managed packages:
```bash
declarch info
```

## Package Issues

### Duplicate Packages Warning

**Symptoms:**
```bash
$ declarch check --duplicates
⚠ Found 2 duplicate package declarations:
  bat (declarch.kdl:5, modules/base.kdl:10)
```

**Cause:** Same package declared multiple times

**Solution:**

Remove duplicate from one location:

```kdl
// declarch.kdl
packages {
    // bat  // Remove from here
    exa
}

// modules/base.kdl
packages {
    bat  // Keep only here
}
```

### Package Name Conflicts

**Symptoms:**
```bash
$ declarch check --conflicts
⚠ Found 1 package name conflicts:
  ripgrep (aur, cargo, soar)
```

**Cause:** Same package name in different backends

**Solution:**

This is a warning, not an error. Decide:

1. **Keep all if intentional** - they install to different locations
2. **Remove from all but one backend**
3. **Check PATH ordering** - determines which one runs

```bash
# Check which one runs
which ripgrep
# /usr/bin/ripgrep  # AUR version
```

### Package Not Adopted

**Symptoms:**
```bash
$ declarch info
Unadopted Packages: 3
  AUR: bat, exa, ripgrep
```

**Cause:** Packages in config but not synced yet

**Solution:**
```bash
declarch sync
```

## State Issues

### "State file corrupted"

**Symptoms:**
```bash
$ declarch sync
✗ Failed to read state file: corrupted data
```

**Cause:** Interrupted sync or corrupted file

**Solution:**

1. Restore from backup:
```bash
cp ~/.local/state/declarch/state.json.backup ~/.local/state/declarch/state.json
```

2. Or start fresh:
```bash
rm ~/.local/state/declarch/state.json
declarch sync  # Will recreate state
```

### State Not Updating

**Symptoms:**
```bash
$ declarch sync
$ declarch info
Managed Packages: 0  # Should show installed packages
```

**Cause:** Sync failed silently

**Solution:**

1. Run with verbose:
```bash
declarch sync -v
```

2. Check for errors
3. Verify permissions:
```bash
ls -la ~/.local/state/declarch/
```

## Backend Issues

### "Backend not available"

**Symptoms:**
```bash
$ declarch check
⚠ Backend 'aur' not available: paru not found
```

**Cause:** Required package manager not installed

**Solution:**

1. Install package manager:
```bash
paru -S paru  # or yay, flatpak, npm, etc.
```

2. Or remove packages from that backend:
```kdl
// Remove aur packages
// packages {
//     hyprland
// }
```

### Custom Backend Not Working

**Symptoms:**
```bash
$ declarch sync --target mybackend
✗ Failed to execute backend command: command not found
```

**Cause:** Backend command not found or wrong path

**Solution:**

1. Test command manually:
```bash
which my-custom-pkg
```

2. Use full path in backend definition:
```kdl
backends {
    custom {
        cmd "/usr/bin/my-custom-pkg install"
    }
}
```

3. See [Custom Backends Guide](custom-backends.md)

## Performance Issues

### Slow Sync

**Symptoms:**
```bash
$ declarch sync
# Takes several minutes
```

**Causes:**
1. Too many packages
2. Slow backend
3. Network latency

**Solutions:**

1. Use targeted sync:
```bash
declarch sync --target flatpak
```

2. Reduce package count
3. Use faster backends (Soar > AUR)

### High Memory Usage

**Symptoms:**
```bash
$ declarch sync
# Uses lots of RAM
```

**Cause:** Large package lists or complex modules

**Solution:**

1. Split into smaller modules
2. Use imports efficiently
3. Report as bug if excessive

## Remote Init Issues

### Repository Not Found

**Symptoms:**
```bash
$ declarch init nonexistent/repo
✗ Failed to fetch configuration: 404 Not Found
```

**Solution:**

1. Verify repository exists:
```bash
curl -I https://github.com/nonexistent/repo
```

2. Check spelling
3. Use direct URL instead:
```bash
declarch init https://example.com/config.kdl
```

### Module Not Found After Remote Init

**Symptoms:**
```bash
$ declarch init myuser/dotfiles
$ declarch check
✗ Failed to resolve import: modules/missing.kdl
```

**Cause:** Remote config references non-existent modules

**Solution:**

1. Report to config maintainer
2. Create missing modules locally
3. Or remove from imports

## Getting Help

### Debug Mode

Get detailed output:

```bash
declarch sync -v
declarch check -v
```

### Dry Run

Preview changes without applying:

```bash
declarch sync --dry-run
```

### Check Configuration

Validate syntax:

```bash
declarch check --verbose --duplicates --conflicts
```

### System Information

Gather info for bug reports:

```bash
declarch info
declarch --version
uname -a
```

### Report Issues

If problems persist:

1. Check [GitHub Issues](https://github.com/nixval/declarch/issues)
2. Search existing issues
3. Create new issue with:
   - Declarch version
   - Error message
   - Configuration file (sanitized)
   - Steps to reproduce

## Recovery

### Full Reset

Start completely fresh:

```bash
# 1. Backup current config
cp -r ~/.config/declarch ~/.config/declarch.backup

# 2. Remove state
rm ~/.local/state/declarch/state.json

# 3. Remove config
rm ~/.config/declarch/declarch.kdl

# 4. Reinitialize
declarch init
```

### Restore Previous State

```bash
# List backups
ls -l ~/.local/state/declarch/

# Restore specific backup
cp ~/.local/state/declarch/state.json.backup.1 ~/.local/state/declarch/state.json
```

## Prevention

### Best Practices

1. **Always check before syncing:**
   ```bash
   declarch check && declarch sync
   ```

2. **Use dry-run before prune:**
   ```bash
   declarch sync --prune --dry-run
   ```

3. **Review remote configs:**
   ```bash
   declarch init user/repo
   cat ~/.config/declarch/declarch.kdl  # Review!
   ```

4. **Keep backups:**
   ```bash
   cp ~/.config/declarch/declarch.kdl ~/.config/declarch/declarch.kdl.bak
   ```

5. **Version control config:**
   ```bash
   cd ~/.config/declarch
   git init
   git add declarch.kdl modules/
   git commit -m "Backup"
   ```

## Related

- [Commands Reference](../commands/) - Command documentation
- [Configuration Guide](../configuration/) - Configuration syntax
- [Advanced Topics](hooks.md) - Hooks and remote init

## Still Having Issues?

1. Check [GitHub Issues](https://github.com/nixval/declarch/issues)
2. Ask in [Discussions](https://github.com/nixval/declarch/discussions)
3. Create new issue with details
