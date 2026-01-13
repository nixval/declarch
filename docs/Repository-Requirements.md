# Repository Requirements for Declarch Init

## Quick Checklist ✅

- [ ] File `declarch.kdl` exists at **repository root**
- [ ] Repository is **public** (or accessible)
- [ ] Branch is `main` or `master` (or specify custom branch)
- [ ] File is **valid KDL syntax**

---

## Detailed Requirements

### 1. File Location: ROOT DIRECTORY

**Required structure:**
```
username/repo/
├── declarch.kdl  ✅ MUST be here
├── README.md
├── LICENSE
└── ...other files
```

**❌ WON'T WORK:**
```
username/repo/
├── configs/
│   └── declarch.kdl  ❌ Wrong location
└── ...
```

```
username/repo/
├── dotfiles/
│   └── declarch.kdl  ❌ Wrong location
└── ...
```

---

### 2. File Name: Exactly `declarch.kdl`

**✅ Correct:**
- `declarch.kdl`

**❌ Wrong:**
- `Declarch.kdl` (capital D)
- `declarch.config.kdl`
- `config.kdl`
- `packages.kdl`

---

### 3. File Format: Valid KDL Syntax

**Minimal valid config:**
```kdl
packages {
    bat
}
```

**More complete:**
```kdl
// My configuration
packages:aur {
    hyprland
}

packages:flatpak {
    com.spotify.Client
}

imports {
    modules/hyprland
}
```

---

### 4. Repository Access

#### Public Repositories (Recommended)
```bash
# Anyone can init
declarch init username/repo
```

#### Private Repositories
Declarch will try to fetch, but will fail if:
- No authentication
- No network access
- Repository doesn't exist

**Workaround:** Use direct URL with authentication token (not recommended)

---

### 5. Branch Naming

Declarch tries branches in this order:

**For `declarch init username/repo`:**
1. `main`
2. `master`

**For `declarch init username/repo/develop`:**
1. `develop` (specified)
2. `main`
3. `master`

**Recommended:** Use `main` (Git default since 2020)

---

## URL Patterns

### GitHub
```
Repository:  username/repo
Branch:      main
File URL:    https://raw.githubusercontent.com/username/repo/main/declarch.kdl

Repository:  username/repo
Branch:      develop
File URL:    https://raw.githubusercontent.com/username/repo/develop/declarch.kdl
```

### GitLab
```
Repository:  gitlab.com/username/repo
Branch:      main
File URL:    https://gitlab.com/username/repo/-/raw/main/declarch.kdl
```

### Direct URL
```bash
declarch init https://any-domain.com/path/to/declarch.kdl
```

---

## Setup Guide

### For GitHub Users

1. **Create repository:**
   ```bash
   mkdir my-dotfiles
   cd my-dotfiles
   git init
   ```

2. **Create declarch.kdl:**
   ```bash
   cat > declarch.kdl << 'EOF'
   packages:aur {
       hyprland
   }
   EOF
   ```

3. **Commit and push:**
   ```bash
   git add declarch.kdl
   git commit -m "Add declarch config"
   git branch -M main
   git remote add origin git@github.com:username/my-dotfiles.git
   git push -u origin main
   ```

4. **Test:**
   ```bash
   # From another machine
   declarch init username/my-dotfiles
   ```

---

### For Projects (Official Configs)

**Example: HyprWM providing official config**

Repository: `https://github.com/hyprwm/hyprland`

1. **Add declarch.kdl to repository root:**
   ```kdl
   // Official Hyprland config for Declarch
   packages:aur {
       hyprland
       waybar
       swww
   }
   ```

2. **Users init:**
   ```bash
   declarch init hyprwm/hyprland
   ```

3. **Document in README:**
   ```markdown
   ## Installation with Declarch
   
   ```bash
   declarch init hyprwm/hyprland
   declarch sync
   ```
   ```

---

## Verification

### Test Your Repository

Before sharing, test if declarch can fetch your config:

```bash
# Test with curl
curl -I https://raw.githubusercontent.com/username/repo/main/declarch.kdl

# Should return: HTTP/1.1 200 OK
```

```bash
# Test with declarch
declarch init username/repo --verbose

# Should see:
# ℹ Trying: https://raw.githubusercontent.com/...
# ✓ Fetched N lines
```

---

## Common Issues

### Issue 1: "Module not found"

**Cause:** File doesn't exist or wrong location

**Check:**
```bash
# Verify file exists
curl https://raw.githubusercontent.com/username/repo/main/declarch.kdl

# If 404, check:
# - File named exactly "declarch.kdl"?
# - At repository root?
# - Repository is public?
# - Branch is "main" (or specify)?
```

**Fix:**
```bash
# Try with explicit branch
declarch init username/repo/master

# Or use direct URL
declarch init https://raw.githubusercontent.com/username/repo/main/declarch.kdl
```

---

### Issue 2: Wrong config fetched

**Cause:** Multiple repos with same name, or wrong file

**Check:**
```bash
declarch edit  # Review fetched config
```

**Fix:** Verify you're fetching from the correct repository

---

### Issue 3: Private repository

**Cause:** Repository is private

**Solution:** Make repository public or use different method

---

## Best Practices

### 1. Use Descriptive Repository Names

**Good:**
- `username/hyprland-config`
- `username/arch-dotfiles`
- `hyprwm/hyprland`

**Bad:**
- `username/config` (too generic)
- `username/stuff` (not descriptive)

---

### 2. Add Documentation to README

```markdown
# My Dotfiles

## Quick Start with Declarch

```bash
declarch init username/my-dotfiles
declarch sync
```

## What's Included

- **Hyprland** - Window manager
- **Waybar** - Status bar
- **CLI tools** - ripgrep, fd, bat, exa

## Manual Installation

See MANUAL_INSTALL.md for traditional setup.
```

---

### 3. Version Your Configs

**Use tags for specific versions:**
```bash
# Tag version 1.0
git tag v1.0.0
git push origin v1.0.0

# Users can pin to version
declarch init https://raw.githubusercontent.com/user/repo/v1.0.0/declarch.kdl
```

---

### 4. Validate Your Config

Before pushing, validate syntax:

```bash
# After creating declarch.kdl
declarch check
```

Or locally:
```bash
# Copy to temp location
cp declarch.kdl ~/.config/declarch/declarch.kdl

# Validate
declarch check
```

---

## Examples

### Example 1: Minimal Config

**Repository:** `username/minimal-config`

**declarch.kdl:**
```kdl
packages:aur {
    hyprland
    waybar
}

packages:soar {
    bat
}
```

**Usage:**
```bash
declarch init username/minimal-config
```

---

### Example 2: Gaming Setup

**Repository:** `gamer123/steam-setup`

**declarch.kdl:**
```kdl
packages:flatpak {
    com.valvesoftware.Steam
    com.discordapp.Discord
}

packages:aur {
    steam
    mangohud
}
```

**Usage:**
```bash
declarch init gamer123/steam-setup
```

---

### Example 3: Development Environment

**Repository:** `devcoder/env`

**declarch.kdl:**
```kdl
packages:npm {
    typescript
    prettier
}

packages:cargo {
    ripgrep
    fd-find
}

packages:python {
    ruff
}
```

**Usage:**
```bash
declarch init devcoder/env
```

---

## Summary

### Requirements:
1. ✅ File named `declarch.kdl` at repository root
2. ✅ Valid KDL syntax
3. ✅ Public repository
4. ✅ Branch: `main` or `master` (or specify)

### Not Required:
- ❌ Specific directory structure
- ❌ Additional metadata files
- ❌ GitHub specifically (GitLab works too)
- ❌ Pull request to declarch-packages

### URLs to Remember:
- GitHub: `https://raw.githubusercontent.com/<user>/<repo>/<branch>/declarch.kdl`
- GitLab: `https://gitlab.com/<user>/<repo>/-/raw/<branch>/declarch.kdl`
