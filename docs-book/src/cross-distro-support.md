# Cross-Distribution Support

declarch is designed to work across different Linux distributions through its official backends and customizable backend system. This document explains distribution support and how to extend declarch with custom package managers.

## Supported Distributions

### Arch-Based Linux (Full Support)

**Distributions:**
- Arch Linux
- EndeavourOS
- Manjaro
- Garuda Linux
- Arco Linux
- Artix Linux

**Available backends:** AUR, Soar, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew

### Debian-Based Linux

**Distributions:**
- Debian
- Ubuntu
- Linux Mint
- Pop!_OS
- elementary OS
- Kali Linux

**Available backends:** Soar, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew

**Note:** AUR packages are not available on Debian-based systems and will be silently ignored.

### Fedora-Based Linux

**Distributions:**
- Fedora
- Red Hat Enterprise Linux (RHEL)
- CentOS
- AlmaLinux
- Rocky Linux

**Available backends:** Soar, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew

**Note:** AUR packages are not available on Fedora-based systems and will be silently ignored.

### Other Distributions

All other Linux distributions support the official backends listed above, plus the ability to add **custom backends** for distribution-specific package managers (see Custom Backends section below).

## Custom Backends

declarch supports unofficial custom backends that you can define in `backend.kdl` files. This allows you to use distribution-specific package managers like:
- **Debian/Ubuntu**: `nala`, `apt`, `apt-get`
- **Fedora/RHEL**: `dnf5`, `dnf`, `yum`
- **openSUSE**: `zypper`
- **Gentoo**: `emerge`
- **Arch alternatives**: `yay`, `paru`

**Important Notes:**
- Custom backends are **unofficial** and not maintained by the declarch team
- The custom backend API is still **fragile** and subject to change
- You are responsible for testing and maintaining custom backend definitions
- Custom backends may not work correctly across all distributions
- Use at your own risk

### Setting Up Custom Backends

To define a custom backend, create a `backend.kdl` file in your config directory:

```kdl
// ~/.config/declarch/backends/nala.kdl
backend "nala" {
  install "nala install -y {packages}"
  remove "nala remove -y {packages}"
  update "nala update -y"
  list "dpkg -l | grep {package}"
  search "nala search {package}"
}
```

Then use it in your packages:

```kdl
// modules/debian-base.kdl
packages:nala {
  vim
  htop
  curl
}
```

### Custom Backend Limitations

- No dependency resolution across backends
- Manual conflict detection required
- Limited package metadata
- May require wrapper scripts for complex operations
- Not tested or guaranteed to work

For production use, prefer official backends (Soar, Flatpak, language-specific managers) whenever possible.

## Official Backend Availability

## Backend Compatibility Matrix

| Backend  | Arch | Debian | Fedora | Other |
|----------|------|--------|--------|-------|
| AUR      | ✓    | ✗      | ✗      | ✗     |
| Soar     | ✓    | ✓      | ✓      | ✓     |
| Flatpak  | ✓    | ✓      | ✓      | ✓     |
| npm      | ✓    | ✓      | ✓      | ✓     |
| yarn     | ✓    | ✓      | ✓      | ✓     |
| pnpm     | ✓    | ✓      | ✓      | ✓     |
| bun      | ✓    | ✓      | ✓      | ✓     |
| pip      | ✓    | ✓      | ✓      | ✓     |
| cargo    | ✓    | ✓      | ✓      | ✓     |
| brew     | ✓    | ✓      | ✓      | ✓     |

## Portable Configuration

To create configurations that work across multiple distributions:

### Use Cross-Distro Backends

```kdl
// Works everywhere
packages:soar {
    bat
    ripgrep
    fd
    exa
}

// Works everywhere
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    com.discordapp.Discord
}

// Works everywhere
packages:npm {
    @types/node
    typescript
    eslint
}
```

### Distro-Specific Backends

```kdl
// Arch-only (will be silently ignored on other distros)
packages {
    hyprland
    waybar
    sway
}

// These packages will only be installed on Arch-based systems
```

### Mixed Configuration

```kdl
// Portable configuration example
packages:soar {
    bat        // Available everywhere
    exa        // Available everywhere
}

packages:flatpak {
    com.spotify.Client  // Available everywhere
}

packages {
    hyprland   // Arch-only, ignored on Debian/Fedora
}
```

## Best Practices

### 1. Prefer Cross-Distro Backends for Portability

When sharing configurations across different machines or distributions, prefer:

- **Soar** for command-line tools
- **Flatpak** for GUI applications
- Language-specific managers (npm, pip, cargo) for development tools

### 2. Use Distro-Specific Backends for Optimization

When creating distribution-specific configurations:

- Use **AUR** for Arch-optimized packages
- Keep these in separate module files
- Document which distro the config is for

### 3. Modular Configuration Structure

```kdl
// declarch.kdl (portable base)
require "base"
require "development"

// modules/arch.kdl (Arch-specific)
packages {
    hyprland
    waybar
}

// modules/universal.kdl (works everywhere)
packages:soar {
    bat
    ripgrep
}
```

### 4. Test Your Configuration

Before sharing your configuration:

1. Test on your target distribution
2. Use `declarch check` to validate syntax
3. Use `declarch info` to see which backends are available
4. Review the list of packages that will be installed

## Automatic Backend Detection

declarch automatically detects your distribution and enables/disables backends accordingly:

- **Arch-based systems:** All backends available
- **Non-Arch systems:** AUR automatically disabled
- **All systems:** Soar, Flatpak, npm, yarn, pnpm, bun, pip, cargo, brew available

No configuration needed - declarch handles this automatically.

## Examples

### Example 1: Portable Dotfiles

```kdl
// Works on Arch, Debian, Fedora, etc.
packages:soar {
    bat
    ripgrep
    fd
    jq
    zoxide
}

packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
    com.slack.Slack
}
```

### Example 2: Arch-Optimized Setup

```kdl
// Arch-specific packages
packages {
    hyprland
    waybar
    sway
    wlogout
}

// Cross-distro packages
packages:soar {
    bat
    ripgrep
}
```

### Example 3: Development Environment

```kdl
// Language-specific tools work everywhere
packages:cargo {
    ripgrep
    fd-find
}

packages:npm {
    @types/node
    typescript
    prettier
}

packages:pip {
    python-lsp-server
    black
}
```

## FAQ

**Q: What happens if I use AUR packages on Debian?**

A: They are silently ignored. No error or warning will be shown. declarch only installs packages from backends that are available on your system.

**Q: Can I force AUR to work on non-Arch systems?**

A: No. AUR is specifically for Arch-based Linux and requires pacman/paru/yay.

**Q: What's the best backend for portable configurations?**

A: Soar for CLI tools and Flatpak for GUI applications. Both work on all distributions.

**Q: Will declarch automatically install the required package managers?**

A: No. You need to have the package managers already installed (flatpak, npm, pip, cargo, etc.). declarch only uses what's available on your system.

**Q: Can I share my declarch config between Arch and Ubuntu?**

A: Yes! Use Soar and Flatpak for packages you want on both systems. Use AUR for Arch-specific packages, and they'll be ignored on Ubuntu.
