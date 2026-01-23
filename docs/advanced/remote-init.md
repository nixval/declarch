# Remote Init

Fetch declarch configurations from Git repositories (GitHub, GitLab, or any URL).

## What Is Remote Init?

Remote init allows you to fetch declarch configurations from any Git repository, similar to Go's package importing:

```bash
declarch init username/dotfiles
```

This makes it easy to:
- Share configurations with others
- Set up new machines quickly
- Try community configurations
- Maintain multiple setups

## Source Formats

| Format | Description | Example |
|--------|-------------|---------|
| `user/repo` | GitHub repository | `myuser/dotfiles` |
| `user/repo:variant` | GitHub with variant | `myuser/dotfiles:hyprland` |
| `user/repo/branch` | GitHub specific branch | `myuser/dotfiles/develop` |
| `user/repo/branch:variant` | Branch + variant | `myuser/dotfiles/main:hyprland` |
| `gitlab.com/user/repo` | GitLab repository | `gitlab.com/myuser/dotfiles` |
| `https://...` | Direct URL | `https://example.com/config.kdl` |
| `registry/module` | Official registry | `hyprland/niri-nico` |

## Basic Usage

### GitHub Repository

```bash
# Fetch default config from repository root
declarch init myuser/dotfiles

# Fetch specific variant
declarch init myuser/dotfiles:hyprland

# Fetch from specific branch
declarch init myuser/dotfiles/develop

# Combine branch and variant
declarch init myuser/dotfiles/main:server
```

### GitLab Repository

```bash
# Fetch from GitLab
declarch init gitlab.com/myuser/dotfiles
declarch init gitlab.com/myuser/dotfiles:hyprland
```

### Direct URL

```bash
# Direct URL to .kdl file
declarch init https://raw.githubusercontent.com/myuser/dotfiles/main/declarch.kdl
declarch init https://example.com/configs/my-setup.kdl
```

### Official Registry

```bash
# Fetch from declarch-packages registry
declarch init hyprland/niri-nico
declarch init gaming/steam-setup
```

## Repository Structure

### Basic Repository

```
myuser/dotfiles/
├── declarch.kdl           # Main config
├── README.md
└── .git
```

Usage:
```bash
declarch init myuser/dotfiles
# Fetches: declarch.kdl
```

### Repository with Variants

```
myuser/dotfiles/
├── declarch.kdl           # Default config
├── declarch-hyprland.kdl  # Hyprland variant
├── declarch-kde.kdl       # KDE variant
├── declarch-server.kdl    # Server variant
└── modules/
    ├── base.kdl
    └── desktop.kdl
```

Usage:
```bash
declarch init myuser/dotfiles              # → declarch.kdl
declarch init myuser/dotfiles:hyprland     # → declarch-hyprland.kdl
declarch init myuser/dotfiles:kde          # → declarch-kde.kdl
declarch init myuser/dotfiles:server       # → declarch-server.kdl
```

### Repository with Branches

```
myuser/dotfiles/ (main branch)
├── declarch.kdl

myuser/dotfiles/ (develop branch)
├── declarch.kdl
├── declarch-experimental.kdl
```

Usage:
```bash
declarch init myuser/dotfiles              # → main branch, declarch.kdl
declarch init myuser/dotfiles/develop      # → develop branch, declarch.kdl
declarch init myuser/dotfiles/develop:exp  # → develop branch, declarch-experimental.kdl
```

## File Resolution Logic

### GitHub

| Input | Resolves To |
|-------|-------------|
| `user/repo` | `https://raw.githubusercontent.com/user/repo/main/declarch.kdl` |
| `user/repo:variant` | `https://raw.githubusercontent.com/user/repo/main/declarch-variant.kdl` |
| `user/repo/branch` | `https://raw.githubusercontent.com/user/repo/branch/declarch.kdl` |
| `user/repo/branch:variant` | `https://raw.githubusercontent.com/user/repo/branch/declarch-variant.kdl` |

### GitLab

| Input | Resolves To |
|-------|-------------|
| `gitlab.com/user/repo` | `https://gitlab.com/user/repo/-/raw/main/declarch.kdl` |
| `gitlab.com/user/repo:variant` | `https://gitlab.com/user/repo/-/raw/main/declarch-variant.kdl` |

### Direct URL

Used as-is:
```bash
declarch init https://example.com/path/to/config.kdl
# Directly fetches that URL
```

## Host-Specific Configs

Use `--host` flag for hostname-specific configurations:

```bash
declarch init myuser/dotfiles --host desktop
```

This looks for:
```
myuser/dotfiles/
├── declarch.kdl              # Default
├── declarch-desktop.kdl      # Host-specific
├── declarch-laptop.kdl
└── declarch-server.kdl
```

## Official Registry

The [declarch-packages](https://github.com/nixval/declarch-packages) repository contains community-maintained configurations:

```bash
# Hyprland setups
declarch init hyprland/niri-nico
declarch init hyprland/hyprland-dots

# Gaming setups
declarch init gaming/steam-setup

# Development setups
declarch init dev/rust-workspace
```

## Creating Your Own Repository

### 1. Create Repository

```bash
mkdir my-dotfiles
cd my-dotfiles
git init
```

### 2. Create Config

```kdl
// declarch.kdl

meta {
    description "My Personal Setup"
    author "yourname"
}

imports {
    modules/base
    modules/desktop
}

packages {
    bat
    exa
    ripgrep
}
```

### 3. Create Modules (Optional)

```bash
mkdir modules
```

```kdl
// modules/base.kdl
packages {
    bat
    exa
    ripgrep
}
```

```kdl
// modules/desktop.kdl
packages {
    hyprland
    waybar
}
```

### 4. Create Variants (Optional)

```bash
# declarch-hyprland.kdl
meta {
    description "Hyprland Desktop Setup"
}

packages {
    hyprland
    waybar
}
```

```bash
# declarch-server.kdl
meta {
    description "Minimal Server Setup"
}

packages {
    vim
    htop
}
```

### 5. Push to GitHub

```bash
git add declarch.kdl declarch-*.kdl modules/
git commit -m "Add declarch config"
git branch -M main
git remote add origin git@github.com:yourname/dotfiles.git
git push -u origin main
```

### 6. Share

Others can now use:
```bash
declarch init yourname/dotfiles
```

Or specific variant:
```bash
declarch init yourname/dotfiles:hyprland
```

## Repository Requirements

For remote init to work, your repository must:

1. **Have a `declarch.kdl` file** at the root (or variant)
2. **Contain valid KDL syntax**
3. **Be publicly accessible** (or have authentication)

### Minimal Repository

```
myuser/dotfiles/
├── declarch.kdl
└── README.md
```

### Complete Repository

```
myuser/dotfiles/
├── README.md                    # Documentation
├── declarch.kdl                 # Default config
├── declarch-hyprland.kdl        # Hyprland variant
├── declarch-kde.kdl             # KDE variant
└── modules/
    ├── base.kdl
    ├── desktop.kdl
    └── development.kdl
```

## Workflow Examples

### Try Community Config

```bash
# 1. Fetch config
declarch init hyprland/niri-nico

# 2. Review it
cat ~/.config/declarch/declarch.kdl

# 3. Check what it would install
declarch check --verbose
declarch sync --dry-run

# 4. Apply
declarch sync
```

### Customize Remote Config

```bash
# 1. Fetch base config
declarch init myuser/dotfiles

# 2. Customize it
declarch edit
# Add/remove packages

# 3. Create your own repo
cd ~/.config/declarch
git init
git add declarch.kdl modules/
git commit -m "My custom setup"

# 4. Push to your repo
git remote add origin git@github.com:yourname/dotfiles.git
git push -u origin main
```

### Multi-Machine Setup

**Desktop:**
```bash
declarch init myuser/dotfiles:desktop --host desktop
declarch sync
```

**Laptop:**
```bash
declarch init myuser/dotfiles:laptop --host laptop
declarch sync
```

**Server:**
```bash
declarch init myuser/dotfiles:server --host server
declarch sync
```

## Security Considerations

### Review Before Applying

Always review remote configs before syncing:

```bash
# 1. Fetch config
declarch init myuser/dotfiles

# 2. Review config file
cat ~/.config/declarch/declarch.kdl

# 3. Check for hooks
cat ~/.config/declarch/declarch.kdl | grep -A 5 "on-"

# 4. Dry run
declarch sync --dry-run

# 5. Apply
declarch sync  # WITHOUT --hooks first
```

### Hooks Are Disabled by Default

Hooks in remote configs are **disabled by default**:

```bash
# Hooks in remote config won't run
declarch sync

# Must explicitly enable
declarch sync --hooks
```

### Trust Sources

Only fetch from sources you trust:
- Known community members
- Official registry
- Friends/colleagues

## Troubleshooting

### Repository Not Found

```bash
$ declarch init nonexistent/repo
✗ Failed to fetch configuration: 404 Not Found
```

**Solution:** Verify repository exists and is public.

### Invalid KDL Syntax

```bash
$ declarch init myuser/dotfiles
✗ Failed to parse configuration: syntax error at line 10
```

**Solution:** Fix syntax in remote repository or report to maintainer.

### Network Issues

```bash
$ declarch init myuser/dotfiles
✗ Failed to fetch configuration: Network error
```

**Solution:** Check internet connection and repository accessibility.

### Module Not Found

```bash
$ declarch init myuser/dotfiles
$ declarch check
✗ Failed to resolve import: modules/missing.kdl
```

**Solution:** Remote config references modules that don't exist. Report to maintainer.

## Best Practices

### For Config Authors

1. **Keep configs modular:** Use imports for organization
2. **Document variants:** Explain each variant in README
3. **Tag configs:** Use meta tags for discoverability
4. **Test configs:** Run `declarch check` before pushing
5. **Avoid dangerous hooks:** Don't require `--hooks` unless necessary

### For Config Users

1. **Always review before syncing**
2. **Use dry-run first**
3. **Fork and customize:** Don't use configs blindly
4. **Give feedback:** Report issues to config authors

## Advanced Patterns

### Base Config with Overrides

```bash
# Start with community config
declarch init hyprland/niri-nico

# Customize locally
declarch edit
# Add your personal packages

# Save to your own repo
cd ~/.config/declarch
git init
git remote add origin git@github.com:yourname/dotfiles.git
git push -u origin main
```

### Layered Configs

```kdl
// declarch.kdl

// Start with community base
imports {
    modules/hyprland/niri-nico  // Symlinked or copied
}

// Add your customizations
packages {
    my-custom-tool
}
```

### Config Discovery

Add to your repository README:

```markdown
# My Dotfiles

## Declarch Configs

- **Default**: Minimal CLI tools
  ```bash
  declarch init myuser/dotfiles
  ```

- **Hyprland**: Wayland desktop with Hyprland
  ```bash
  declarch init myuser/dotfiles:hyprland
  ```

- **KDE**: KDE Plasma desktop
  ```bash
  declarch init myuser/dotfiles:kde
  ```

## Features

- 🎨 Hyprland window manager
- 🚀 Fast CLI tools (bat, exa, ripgrep)
- 📦 Flatpak apps
- 🔧 Development tools
```

## Related

- [Init Command](../commands/init.md) - Initialize command reference
- [Repository Requirements](../reference/repo-requirements.md) - Setting up your repo
- [Modules Guide](../configuration/modules.md) - Modular configuration

## See Also

- [declarch-packages Registry](https://github.com/nixval/declarch-packages)
- [Creating Community Configs](../reference/repo-requirements.md)

## Examples

### Official Registry Configs

```bash
# Browse available configs
# https://github.com/nixval/declarch-packages

# Use a config
declarch init hyprland/niri-nico
```

### Personal Configs

```bash
# Share your config
git push yourname/dotfiles

# Others use it
declarch init yourname/dotfiles
```

### Team/Company Configs

```bash
# Team standard setup
declarch init gitlab.com/company/team-dev-env
```
