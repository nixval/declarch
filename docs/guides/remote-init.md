# Declarch Remote Init Guide

## Overview

Declarch supports fetching KDL configurations from **any** Git repository, similar to Go's package importing. This allows users and projects to host their own dotfile configurations without contributing to the central declarch-packages registry.

---

## Usage Examples

### 1. Official Registry (Default)

```bash
# Fetch from declarch-packages
declarch init hyprland/niri-nico
```

**Fetches from:**
```
https://raw.githubusercontent.com/nixval/declarch-packages/main/modules/hyprland/niri-nico.kdl
```

---

### 2. Arbitrary GitHub Repository (Go-style)

```bash
# Fetch from user's GitHub repo
declarch init myuser/hyprland1
declarch init hyprwm/hyprland
declarch init username/dotfiles
```

**Fetches from:**
```
https://raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl
https://raw.githubusercontent.com/hyprwm/hyprland/main/declarch.kdl
https://raw.githubusercontent.com/username/dotfiles/main/declarch.kdl
```

**With specific branch:**
```bash
declarch init username/repo/develop
```

**Tries branches in order:**
1. `develop` (specified)
2. `main`
3. `master`

---

### 3. GitLab Repository

```bash
declarch init gitlab.com/user/repo
declarch init gitlab.com/user/dotfiles/develop
```

**Fetches from:**
```
https://gitlab.com/user/repo/-/raw/main/declarch.kdl
https://gitlab.com/user/dotfiles/-/raw/develop/declarch.kdl
```

---

### 4. Direct URL

```bash
declarch init https://example.com/config.kdl
declarch init https://raw.githubusercontent.com/user/repo/v1.0/config.kdl
```

---

## Repository Setup

### For Users (Personal Dotfiles)

**1. Create a GitHub repository with your dotfiles:**

```
my-dotfiles/
├── README.md
├── declarch.kdl    ← Root configuration
└── modules/        ← Optional modules
    ├── hyprland.kdl
    └── gaming.kdl
```

**2. Add `declarch.kdl` to your repository:**

```kdl
// declarch.kdl

// My personal Hyprland setup
packages {
    aur {
        hyprland
        waybar
        swww
    }
}

// System packages
packages:aur {
    alacritty
    zathura-pdf-mupdf
}

// CLI tools
packages:soar {
    bat
    exa
    fd
    ripgrep
}
```

**3. Users can now init your config:**

```bash
declarch init yourusername/my-dotfiles
```

---

### For Projects (Official Configs)

**Example: HyprWM providing official config**

**Repository structure:**
```
hyprland/
├── declarch.kdl     ← Official Hyprland config
├── README.md
├── src/
└── ...
```

**`declarch.kdl` content:**
```kdl
// Official Hyprland package configuration for Declarch
// Source: https://github.com/hyprwm/hyprland

packages:aur {
    // Core Hyprland
    hyprland

    // Essential addons
    waybar
    swww
    wlogout
    hyprpaper
    hyprpicker
}

packages:flatpak {
    // Recommended flatpaks
    org.freedesktop.Platform.VulkanLayer
}
```

**Users initialize:**
```bash
declarch init hyprwm/hyprland
```

---

## URL Resolution Logic

Declarch tries multiple URL patterns in order:

```bash
declarch init myuser/hyprland1
```

**Tries in order:**
1. `https://raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl`
2. `https://raw.githubusercontent.com/myuser/hyprland1/master/declarch.kdl`
3. `https://raw.githubusercontent.com/nixval/declarch-packages/main/modules/myuser/hyprland1.kdl`

**Stops at first successful fetch.**

---

## Best Practices

### 1. Repository Naming

**Good:**
- `my-dotfiles`
- `hyprland-config`
- `arch-setup`

**Bad:**
- `config` (too generic)
- `stuff` (not descriptive)

### 2. File Location

**Always put `declarch.kdl` at repository root:**
```
my-config/
├── declarch.kdl  ✅
├── README.md
└── modules/
```

**NOT:**
```
my-config/
└── configs/
    └── declarch.kdl  ❌ Won't be found
```

### 3. Documentation

**Add usage instructions to your README:**

```markdown
# My Dotfiles

## Installation with Declarch

```bash
declarch init myuser/my-dotfiles
declarch sync
```

## Manual Installation

```bash
git clone https://github.com/myuser/my-dotfiles.git
cd my-dotfiles
# ... manual setup
```
```

### 4. Branch Selection

**Use standard branch names:**
- `main` (recommended)
- `master` (legacy)

**For custom branches:**
```bash
declarch init username/repo/develop
```

---

## Examples

### Gaming Setup

```bash
# Fetch gaming-focused config
declarch init gamer123/steam-setup
```

**Repository content:**
```kdl
packages:flatpak {
    com.valvesoftware.Steam
    com.discordapp.Discord
    org.prismlauncher.PrismLauncher
}

packages:aur {
    steam
    mangohud
    gamemode
}
```

### Development Environment

```bash
# Fetch dev-focused config
declarch init developer/dev-environment
```

**Repository content:**
```kdl
packages:npm {
    typescript
    prettier
    eslint
}

packages:cargo {
    ripgrep
    fd-find
    eza
    git-delta
}

packages:python {
    ruff
    black
}
```

---

## Migration from Central Registry

### Old Way (Still Supported)

```bash
declarch init hyprland/niri-nico
# From: github.com/nixval/declarch-packages
```

### New Way (More Flexible)

```bash
# From your own repo
declarch init myuser/hyprland1

# From official project repo
declarch init hyprwm/hyprland

# From direct URL
declarch init https://example.com/config.kdl
```

---

## Troubleshooting

### "Module not found" Error

**Problem:**
```bash
declarch init myuser/hyprland1
# Error: Failed to fetch from: myuser/hyprland1
```

**Solutions:**
1. **Check repository exists:**
   ```bash
   curl -I https://github.com/myuser/hyprland1
   ```

2. **Check file exists:**
   ```bash
   curl -I https://raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl
   ```

3. **Try with explicit branch:**
   ```bash
   declarch init myuser/hyprland1/master
   ```

4. **Use direct URL:**
   ```bash
   declarch init https://raw.githubusercontent.com/myuser/hyprland1/develop/declarch.kdl
   ```

### Wrong Config Fetched

**Problem:** Repository has `declarch.kdl` but it's not the one you want.

**Solution:** Use specific module path:
```kdl
// In your root declarch.kdl
imports {
    myuser/my-config/modules/hyprland
}
```

---

## Comparison: Go vs Declarch

| Go | Declarch | Purpose |
|----|----------|---------|
| `go get github.com/user/repo` | `declarch init user/repo` | Fetch from GitHub |
| `golang.org/x/tools` | `declarch/packages` | Official registry |
| `gitlab.com/user/repo` | `gitlab.com/user/repo` | GitLab support |
| Custom import paths | Direct URLs | Arbitrary sources |

---

## Advanced: Multiple Sources

You can combine configs from multiple sources:

```kdl
// declarch.kdl

// Local customizations
packages:aur {
    my-custom-package
}

// Import from GitHub user's config
imports {
    myuser/hyprland1
    gamer123/steam-setup
}

// Import from official project
imports {
    hyprwm/hyprland
}
```

---

## Security Considerations

### 1. Verify Source

Always verify the repository before initializing:

```bash
# Check the repository first
gh repo view myuser/hyprland1
# or
xdg-open https://github.com/myuser/hyprland1
```

### 2. Review Config

Declarch shows the fetched content:

```bash
declarch init myuser/hyprland1
# Output:
# ℹ Fetching from: https://raw.githubusercontent.com/...
# ✓ Fetched 45 lines
```

Review the file before syncing:

```bash
declarch edit  # Review the fetched config
declarch check  # Verify syntax
declarch sync  # Apply
```

### 3. Use Specific Versions

For reproducibility, consider:

```bash
# Tag-specific URL
declarch init https://raw.githubusercontent.com/user/repo/v1.0.0/declarch.kdl
```

---

## Summary

✅ **Pros:**
- No need to PR to central registry
- Projects can provide official configs
- Users maintain their own configs
- Familiar Go-style syntax
- GitLab, GitHub, and direct URLs supported

❌ **Cons:**
- Requires internet connection
- Depends on external repository availability
- Security consideration (verify source)

🎯 **Best For:**
- Personal dotfiles
- Project-specific configs
- Community-maintained setups
- Quick bootstrapping

---

## See Also

- [Declarch README](https://github.com/nixval/declarch)
- [Syntax Guide](./SYNTAX.md)
- [Contributing](https://github.com/nixval/declarch-packages)
