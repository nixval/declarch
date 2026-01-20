# Release Notes

## [0.4.1] - 2025-01-20

### Major Features

**🎨 User-Defined Backends - Extensible Package Manager Support**
- Define custom package managers via KDL configuration
- No code changes required to add new backends
- Support for any package manager with text/JSON output
- Configuration location: `~/.config/declarch/backends.kdl`

**🔧 Multiple Output Format Parsers**
- JSON parser with nested path support (e.g., nala, dnf5, npm)
- TSV parser for tab-separated output (e.g., zypper, flatpak)
- Whitespace parser for space-separated output (e.g., pacman, cargo)
- Regex parser for complex patterns (e.g., apt)

**🔍 Backend Filtering in Check Command**
- New `--backend` flag for `declarch check`
- Filter packages by specific backend: `declarch check --backend nala --verbose`
- Supports both built-in and user-defined backends

### Production-Ready Backend Examples

**NALA (Debian/Ubuntu)** - JSON format
```kdl
backend "nala" {
    binary "nala"
    list "nala list --installed --json" {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }
    install "nala install -y {packages}"
    remove "nala remove -y {packages}"
    noconfirm "-y"
    needs_sudo true
}
```

**Zypper (openSUSE)** - TSV format
```kdl
backend "zypper" {
    binary "zypper"
    list "zypper search --installed-only --type package --details" {
        format tsv
        name_col 2
        version_col 3
    }
    install "zypper install --no-confirm {packages}"
    remove "zypper remove --no-confirm {packages}"
    noconfirm "--no-confirm"
    needs_sudo true
}
```

**DNF5 (Fedora)** - JSON format
```kdl
backend "dnf5" {
    binary ["dnf5", "dnf"]
    list "dnf5 list installed --json" {
        format json
        json_path "packages"
        name_key "name"
        version_key "version"
    }
    install "dnf5 install -y {packages}"
    remove "dnf5 remove -y {packages}"
    noconfirm "-y"
    needs_sudo true
}
```

**APT (Debian/Ubuntu)** - Regex format
```kdl
backend "apt" {
    binary "apt"
    list "apt list --installed" {
        format regex
        regex "^([^/]+)\\s+([^\\s]+)\\s+"
        name_group 1
        version_group 2
    }
    install "apt install --yes {packages}"
    remove "apt remove --yes {packages}"
    noconfirm "--yes"
    needs_sudo true
}
```

**Pacman (Arch)** - Whitespace format
```kdl
backend "pacman" {
    binary "pacman"
    list "pacman -Q" {
        format whitespace
        name_col 0
        version_col 1
    }
    install "pacman -S --noconfirm {packages}"
    remove "pacman -R --noconfirm {packages}"
    noconfirm "--noconfirm"
    needs_sudo true
}
```

### Code Quality Improvements

**✅ Clippy Compliance**
- All 26 clippy warnings resolved
- Passes strict CI: `cargo clippy --all-targets -- -D warnings`
- Idiomatic Rust 2024 patterns:
  - Let-chains for nested conditionals
  - Struct initialization instead of default+assign
  - `or_default()` instead of `or_insert_with(Vec::new)`
  - `.values()` instead of `for (_key, val)`
  - `is_some()` instead of `!is_none()`

**✅ Test Coverage**
- 141 tests passing (including new backend parser tests)
- Integration tests for JSON, TSV, and whitespace parsers
- All edge cases covered

### Backend System Enhancements

- **Custom Backend Support**: New `Backend::Custom(String)` variant
- **Dynamic Backend Loading**: Load from `~/.config/declarch/backends.kdl`
- **Fallback Binaries**: Support multiple binary alternatives (e.g., `["dnf5", "dnf"]`)
- **Environment Variables**: Set per-backend environment variables
- **Flexible Parsing**: 4 output formats with extensible configuration

### Documentation

- **User-Defined Backends Guide**: Comprehensive documentation with tested examples
- **Troubleshooting Section**: Common issues and solutions
- **Best Practices**: Testing checklist and validation guide
- **Advanced Template**: For creating custom backends

### Breaking Changes

None. This is a feature release that maintains backward compatibility.

### Migration from 0.4.0

No migration needed. User-defined backends are opt-in via KDL configuration.

### Contributors

- Code by: Claude Sonnet 4.5
- Directed by: nixval

---

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
a9bfc9c3d8e8184a11b1e61ee6fee3d6a5e631a783206db021c13993ccf19882  declarch-0.4.0.tar.gz
```

**Binary Release (x86_64):**
```
0d022d1dc36213959e816201f77adf9170ed8360e76c52113f4c3ac4  declarch-x86_64-unknown-linux-gnu.tar.gz
```

### Contributors

- @nixval

### Links

- **GitHub**: https://github.com/nixval/declarch
- **AUR**: https://aur.archlinux.org/packages/declarch
- **Wiki**: https://github.com/nixval/declarch/wiki
- **Issues**: https://github.com/nixval/declarch/issues
