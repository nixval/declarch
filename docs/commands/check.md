# declarch check

Validate configuration syntax and check for issues.

## Usage

```bash
declarch check [OPTIONS]
```

## Description

The `check` command validates your declarch configuration without making any changes to your system. It:

1. Parses and validates KDL syntax
2. Resolves all imports
3. Checks for duplicate packages
4. Detects cross-backend package conflicts
5. Shows resolved packages per backend

## Options

| Option | Description |
|--------|-------------|
| `--verbose` | List all resolved packages |
| `--duplicates` | Check for duplicate package declarations |
| `--conflicts` | Check for cross-backend package name conflicts |
| `--backend <BACKEND>` | Validate only specific backend |

## Examples

### Basic Check

```bash
declarch check
```

Output:
```
=== Configuration Check ===
✓ Configuration file exists
✓ KDL syntax valid
✓ All imports resolved
✓ No duplicates found
✓ No conflicts found

=== Resolved Packages ===
AUR: 3 packages
Flatpak: 2 packages
npm: 2 packages
Total: 7 packages
```

### Verbose Output

```bash
declarch check --verbose
```

Output:
```
=== Configuration Check ===
✓ Configuration file exists
✓ KDL syntax valid
✓ All imports resolved

=== Resolved Packages ===

AUR (3):
  - hyprland
  - waybar
  - wofi

Flatpak (2):
  - com.spotify.Client
  - org.mozilla.firefox

npm (2):
  - typescript
  - prettier

Total: 7 packages
```

### Check for Duplicates

```bash
declarch check --duplicates
```

Output:
```
=== Configuration Check ===
✓ Configuration file exists
✓ KDL syntax valid
✓ All imports resolved

⚠ Found 2 duplicate package declarations:

  hyprland
    ├─ declarch.kdl:15
    └─ modules/desktop.kdl:8

  waybar
    ├─ declarch.kdl:16
    └─ modules/desktop.kdl:9
```

### Check for Conflicts

```bash
declarch check --conflicts
```

Output:
```
=== Configuration Check ===
✓ Configuration file exists
✓ KDL syntax valid
✓ All imports resolved

⚠ Found 1 package name conflicts across backends:

These packages have the same name but different backends:
They will be installed separately by each backend.
Watch out for PATH conflicts!

  ⚠️ ripgrep
     ├─ aur
     ├─ cargo
     └─ soar
```

### Check Specific Backend

```bash
declarch check --backend aur
```

Only validates AUR packages.

### All Checks

```bash
declarch check --verbose --duplicates --conflicts
```

Run all checks with detailed output.

## What Gets Checked

### 1. File Existence

Verifies configuration file exists:
```
✓ Configuration file exists: ~/.config/declarch/declarch.kdl
```

### 2. KDL Syntax

Validates KDL syntax:
```
✓ KDL syntax valid
```

**Example error:**
```
✗ KDL syntax error:
   → Line 25: Expected identifier, found '}'

    24 │     packages {
    25 │         bat exa
    26 │     }
```

### 3. Import Resolution

Resolves all module imports:
```
✓ All imports resolved
```

**Example error:**
```
✗ Failed to resolve import:
   → modules/missing.kdl: No such file or directory
```

### 4. Duplicate Detection

Finds packages declared multiple times:
```
⚠ Found 2 duplicate package declarations:
  bat (declared in declarch.kdl:5, modules/base.kdl:10)
```

### 5. Conflict Detection

Finds packages with same name across different backends:
```
⚠ Found 1 package name conflicts:
  ripgrep (exists in aur, cargo, soar)
```

### 6. Backend Validation

Validates backend-specific requirements:
```
✓ AUR backend available
✓ Flatpak backend available
✓ npm backend available
```

**Example error:**
```
⚠ Backend 'aur' not available: paru not found
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | Errors found (syntax, imports) |
| 2 | Warnings found (duplicates, conflicts) |

Use in scripts:
```bash
declarch check
if [ $? -eq 0 ]; then
    declarch sync
fi
```

## Use Cases

### Before Syncing

Always check before syncing:
```bash
declarch check
declarch sync --dry-run
declarch sync
```

### After Editing Config

Validate syntax immediately after editing:
```bash
declarch edit
declarch check
```

### CI/CD Integration

Use in automated workflows:
```bash
#!/bin/bash
declarch check --duplicates --conflicts
if [ $? -ne 0 ]; then
    echo "Configuration has errors"
    exit 1
fi
declarch sync --noconfirm
```

### Debugging Configuration Issues

Find problematic packages:
```bash
# Check for duplicates
declarch check --duplicates

# Check for conflicts
declarch check --conflicts

# List all packages
declarch check --verbose
```

## Troubleshooting

### "KDL syntax error"

**Cause:** Invalid KDL syntax in configuration

**Solution:**
1. Note the line number from error message
2. Check for:
   - Missing closing braces `}`
   - Invalid characters
   - Incorrect nesting
3. Fix and re-run `declarch check`

Example:
```kdl
// ❌ Wrong
packages {
    bat exa

// ✅ Correct
packages {
    bat
    exa
}
```

### "Failed to resolve import"

**Cause:** Module file doesn't exist

**Solution:**
```bash
# Check if file exists
ls ~/.config/declarch/modules/missing.kdl

# Create it or remove import
declarch edit
```

### "Backend not available"

**Cause:** Required package manager not installed

**Solution:**
```bash
# Install missing package manager
paru -S flatpak  # or npm, pip, cargo, etc.

# Or remove packages from that backend
declarch edit
```

### "Duplicate package declarations"

**Cause:** Same package declared multiple times

**Solution:**
1. Remove duplicate from config or module
2. Or keep it in one location only

Example:
```kdl
// declarch.kdl
packages {
    bat  // ← Remove this
}

// modules/base.kdl
packages {
    bat  // ← Keep only here
}
```

### "Package name conflicts"

**Cause:** Same package name in different backends

**Solution:**
This is a warning, not an error. Decide:
1. Keep all if intentional (different binaries)
2. Remove from all but one backend
3. Use backend-specific names if needed

Example:
```kdl
// All three will install separately
packages {
    ripgrep  // AUR: /usr/bin/ripgrep
}

packages:cargo {
    ripgrep  // Cargo: ~/.cargo/bin/ripgrep
}

packages:soar {
    ripgrep  // Soar: ~/.local/bin/ripgrep
}
```

Your PATH ordering determines which one runs!

## Related Commands

- [`edit`](edit.md) - Edit configuration files
- [`sync`](sync.md) - Apply configuration
- [`info`](info.md) - Show system status

## Tips

1. **Always check after editing:**
   ```bash
   declarch edit && declarch check
   ```

2. **Use verbose mode to review packages:**
   ```bash
   declarch check --verbose
   ```

3. **Check for conflicts before syncing:**
   ```bash
   declarch check --conflicts
   ```

4. **Use in scripts:**
   ```bash
   declarch check && declarch sync
   ```

5. **Validate specific backends:**
   ```bash
   declarch check --backend aur
   ```
