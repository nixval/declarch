# Release Notes

## [0.4.0] - 2025-01-13

### Major Features

**🎯 Remote Init - Fetch Configs from GitHub/GitLab**
- Support Go-style package importing: `declarch init user/repo`
- Fetch from arbitrary GitHub repositories
- Support GitLab repositories: `declarch init gitlab.com/user/repo`
- Direct URL support: `declarch init https://example.com/config.kdl`
- Community registry fallback: declarch-packages integration

**🔢 Config Variants - Multiple Configs in One Repository**
- Config variant syntax: `user/repo:variant`
- Fetch specific config: `declarch init myuser/dotfiles:uwsm`
- Branch + variant: `declarch init myuser/dotfiles/develop:uwsm`
- Example repository can have multiple config files:
  - `declarch.kdl` (default)
  - `declarch-uwsm.kdl`
  - `declarch-minimal.kdl`

**✏️ Editor Configuration**
- Set editor in KDL: `editor "nvim"`
- Priority: KDL config → $EDITOR → $VISUAL → nano
- Used by `declarch edit` command

**📝 Description Syntax**
- Add description nodes for documentation: `description { ... }`
- No-op nodes (doesn't affect package installation)
- Purely for documentation purposes

**🌐 Cross-Distro Support via Soar**
- Soar backend: Static binaries (works on all Linux)
- AUR backend: Arch-based distros only
- Flatpak backend: Universal (all Linux with Flatpak)
- Flexible syntax: `packages`, `packages:aur`, `packages:flatpak`

**🔧 Edit Command**
- Edit config in your preferred editor
- Syntax verification after editing
- Opens root config or specific modules

**🔄 Switch Command**
- Replace one package with another
- Useful for variants: `firefox` → `firefox-beta`
- Supports all backends

### Syntax Improvements

**Flexible Packages Syntax**
```kdl
// Method 1: Colon syntax
packages:aur {
    hyprland
}

// Method 2: Embedded
packages {
    bat
    aur {
        hyprland
    }
}
```

### Documentation

- **GitHub Wiki**: 7 comprehensive documentation pages
  - Home, Installation, Quick-Start
  - KDL Syntax Reference
  - Remote Init Guide
  - Repository Requirements
  - Examples
- **README.md**: Updated with v0.4.0 features
- **Cross-distro**: Positioning updated from "Arch-only" to "Linux package manager"

### Bug Fixes

- Fix double `.kdl.kdl` extension bug in registry paths
- Fix URL building for direct URLs (preserve extension)
- Fix aggressive prefix matching bug (polkit/polkit-kde-agent)

### Internal Changes

- Refactor `src/utils/remote.rs`: Enhanced URL resolution logic
- Add `src/commands/edit.rs`: New edit command
- Add `src/commands/switch.rs`: New switch command
- Refactor `src/config/kdl.rs`: Parse editor and description nodes
- Update `src/core/types.rs`: Backend improvements
- Add shell completions: bash, zsh, fish
- Add `.aur/` directory: AUR publish scripts

### Breaking Changes

None. Fully backward compatible with v0.3.x configs.

### Migration from v0.3.x

No migration needed. Existing configs work as-is.

### Installation

```bash
# From AUR (recommended)
paru -S declarch

# Binary
paru -S declarch-bin

# Install script
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### Upgrade

```bash
# If installed from AUR
paru -Syu declarch

# If installed via binary
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install-remote.sh | sh
```

###SHA256 Checksums

**Source Tarball:**
```
57cec80b165c200b27a1f04f4e6c9575586ea713f1caf104cdb3a662288900fe  declarch-0.4.0.tar.gz
```

### Contributors

- @nixval

### Links

- **GitHub**: https://github.com/nixval/declarch
- **AUR**: https://aur.archlinux.org/packages/declarch
- **Wiki**: https://github.com/nixval/declarch/wiki
- **Issues**: https://github.com/nixval/declarch/issues
