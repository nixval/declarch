# declarch init

Initialize a new declarch configuration or fetch one from a remote repository.

## Usage

```bash
declarch init [SOURCE] [OPTIONS]
declarch init [FLAGS]
```

## Arguments

### SOURCE (Optional)

Config source to fetch. If not provided, creates a new empty configuration.

| Format | Description | Example |
|--------|-------------|---------|
| `user/repo` | GitHub repository | `myuser/dotfiles` |
| `user/repo:variant` | GitHub with config variant | `myuser/dotfiles:hyprland` |
| `user/repo/branch` | GitHub specific branch | `myuser/dotfiles/develop` |
| `user/repo/branch:variant` | Branch + variant | `myuser/dotfiles/main:hyprland` |
| `gitlab.com/user/repo` | GitLab repository | `gitlab.com/myuser/dotfiles` |
| `https://...` | Direct URL to `.kdl` file | `https://example.com/config.kdl` |
| `registry/module` | Official registry | `hyprland/niri-nico` |

## Options

| Option | Description |
|--------|-------------|
| `--host <NAME>` | Hostname-specific config |
| `--skip-soar-install` | Skip automatic Soar installation |

## Examples

### Create New Empty Configuration

```bash
declarch init
```

Creates:
- `~/.config/declarch/declarch.kdl` - Main config
- `~/.local/state/declarch/state.json` - Package state

### Fetch from GitHub

```bash
# Fetch default config from repository root
declarch init myuser/dotfiles

# Fetch specific variant
declarch init myuser/dotfiles:hyprland

# Fetch from specific branch
declarch init myuser/dotfiles/develop

# Branch + variant
declarch init myuser/dotfiles/main:server
```

### Fetch from GitLab

```bash
declarch init gitlab.com/myuser/dotfiles
declarch init gitlab.com/myuser/dotfiles:hyprland
```

### Direct URL

```bash
declarch init https://raw.githubusercontent.com/myuser/dotfiles/main/declarch.kdl
declarch init https://example.com/configs/my-setup.kdl
```

### Official Registry

```bash
declarch init hyprland/niri-nico
declarch init gaming/steam-setup
```

### Host-Specific Configuration

```bash
# Fetch config for specific hostname
declarch init myuser/dotfiles --host desktop
```

This looks for `declarch-desktop.kdl` in the repository.

## What Happens During Init

### Without SOURCE (Local Init)

1. Creates config directory: `~/.config/declarch/`
2. Creates default config: `declarch.kdl`
3. Creates state file: `~/.local/state/declarch/state.json`
4. Creates module directory: `~/.config/declarch/modules/`
5. Optionally installs Soar (unless `--skip-soar-install`)

### With SOURCE (Remote Init)

1. Resolves source URL:
   - GitHub: `https://raw.githubusercontent.com/user/repo/branch/declarch[-variant].kdl`
   - GitLab: `https://gitlab.com/user/repo/-/raw/branch/declarch[-variant].kdl`
   - Direct: Uses URL as-is
   - Registry: Fetches from `declarch-packages` repository

2. Downloads configuration file

3. Creates directory structure:
   - `~/.config/declarch/declarch.kdl` (main config)
   - Modules if referenced in imports

4. Initializes state file

## Repository Requirements

For remote init to work, the repository must:

1. Have a `declarch.kdl` file at the root
2. Have valid KDL syntax
3. Be publicly accessible (or authenticated)

### Example Repository Structure

```
myuser/dotfiles/
├── declarch.kdl           # Main config (default)
├── declarch-hyprland.kdl  # Variant: hyprland
├── declarch-server.kdl    # Variant: server
└── modules/
    ├── base.kdl
    ├── desktop.kdl
    └── development.kdl
```

Usage:
```bash
declarch init myuser/dotfiles              # → declarch.kdl
declarch init myuser/dotfiles:hyprland     # → declarch-hyprland.kdl
declarch init myuser/dotfiles:server       # → declarch-server.kdl
```

## Post-Init Steps

After initializing:

```bash
# 1. Review the config
cat ~/.config/declarch/declarch.kdl

# 2. Edit if needed
declarch edit

# 3. Validate
declarch check

# 4. Preview changes
declarch sync --dry-run

# 5. Apply
declarch sync
```

## Common Patterns

### Multi-Machine Setup

```bash
# On desktop
declarch init myuser/dotfiles:desktop --host desktop

# On laptop
declarch init myuser/dotfiles:laptop --host laptop

# On server
declarch init myuser/dotfiles:server --host server
```

### Community Configs

```bash
# Hyprland setup
declarch init hyprland/niri-nico

# Gaming setup
declarch init gaming/steam-setup

# Development setup
declarch init dev/rust-workspace
```

### Personal Dotfiles

Share your own configuration:

1. Push `declarch.kdl` to your dotfiles repo
2. Share with others:
   ```bash
   declarch init yourusername/dotfiles
   ```

## Error Handling

### Repository Not Found

```bash
declarch init nonexistent/repo
# Error: Failed to fetch configuration: 404 Not Found
```

**Solution:** Verify repository URL and that it contains `declarch.kdl`

### Invalid KDL Syntax

```bash
declarch init myuser/dotfiles
# Error: Failed to parse configuration: syntax error at line 10
```

**Solution:** Fix syntax in remote config, or report to maintainer

### Network Issues

```bash
declarch init myuser/dotfiles
# Error: Failed to fetch configuration: Network error
```

**Solution:** Check internet connection and repository accessibility

## Tips

1. **Always review remote configs** before syncing:
   ```bash
   declarch init myuser/dotfiles
   cat ~/.config/declarch/declarch.kdl  # Review!
   declarch check
   declarch sync --dry-run
   ```

2. **Use variants for different setups**:
   - `:hyprland` - Hyprland desktop
   - `:server` - Minimal server setup
   - `:work` - Work-specific packages

3. **Version control your config**:
   ```bash
   cd ~/.config/declarch
   git init
   git add declarch.kdl modules/
   git commit -m "Initial declarch config"
   ```

## Related Commands

- [`edit`](edit.md) - Edit configuration files
- [`check`](check.md) - Validate configuration
- [`sync`](sync.md) - Apply configuration

## See Also

- [Remote Init Guide](../advanced/remote-init.md) - Detailed remote init documentation
- [Repository Requirements](../reference/repo-requirements.md) - Setting up your own repository
