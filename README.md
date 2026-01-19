
# 🌱 **declarch**

<p align="center">
<strong>A declarative package manager for Linux — powered by Rust.</strong><br>
Inspired by Nix workflow, supporting AUR, Flatpak, and Soar.
</p>


<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Build" src="https://img.shields.io/badge/status-v0.4.0-orange">
  <img alt="Arch" src="https://img.shields.io/badge/arch-linux-blue">
  <img alt="Rust" src="https://img.shields.io/badge/built_with-rust-orange">
</p>

-----

## 🧠 The Philosophy

Linux package management is often **imperative**. You run `pacman -S git`, `apt install git`, or `dnf install git`, and then you forget about it. Over time, your system becomes a "museum" of forgotten packages, orphans, and drift.

```kdl
// ~/.config/declarch/declarch.kdl

// AUR packages (Arch Linux)
packages:aur {
    hyprland
    waybar
}

// Static binaries (Soar)
packages {
    bat
    exa
    ripgrep
}

// Flatpak applications
packages:flatpak {
    com.spotify.Client
}

// Node.js global packages
packages:npm {
    typescript
    prettier
}

// Python packages
packages:pip {
    ruff
    black
}

// Rust crates
packages:cargo {
    ripgrep
    fd-find
}
```

Then simply:

```bash
declarch sync
```

Share your config across different Linux distributions.

**declarch** imposes a **Declarative Layer** on top of existing package managers.

1.  **Intent vs. State:** You declare *what* you want in a `.kdl` file. `declarch` ensures your system matches that state.
2.  **Adoption, Not Reinstallation:** If you declare `vim` and it's already installed, `declarch` simply "adopts" it.
3.  **Performance:** Uses smart batching to check hundreds of packages instantly.
4.  **Safe Pruning:** Only removes packages that it *knows* it manages.

-----

## ✨ Key Features

  * **Declarative Config:** Uses the clean, readable **KDL** syntax.
  * **Dual Command:** Use `declarch` or shorter `dc` alias.
  * **Remote Init:** Fetch configs from GitHub/GitLab repositories.
  * **Universal Backend:** Supports **AUR**, **Flatpak**, **Soar**, **npm**, **pip**, **cargo**, **brew**, **yarn**, **pnpm**, **bun**.
  * **Generic System:** Easy to add new package managers via configuration.
  * **Flexible Syntax:** Write packages your way — simple, nested, or mixed.
  * **Modular:** Import and organize configs into reusable modules.
  * **Smart Sync:** Auto-installs missing packages, adopts existing ones.
  * **Advanced Config:** Meta info, conflicts, env vars, hooks, policy control.

-----

## 🚀 Installation

### From AUR (Recommended)

```bash
paru -S declarch
```

Or install pre-built binary:

```bash
paru -S declarch-bin
```

### Install Script

Downloads the latest binary and sets up the environment.

```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### Build from Source

```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

-----

## 📁 Getting Started

### Initialize Config

```bash
# Using declarch
declarch init

# Or use the shorter alias
dc init
```

This creates:
- `~/.config/declarch/declarch.kdl` - Main configuration
- `~/.config/declarch/modules/base.kdl` - Base system packages

The default config includes all advanced syntax features (commented out) for easy discovery.

### Fetch from Remote

```bash
# From GitHub user repository
declarch init myuser/dotfiles

# From official project config
declarch init hyprwm/hyprland

# From community registry
declarch init hyprland/niri-nico

# From GitLab
declarch init gitlab.com/user/repo

# Direct URL
declarch init https://example.com/config.kdl
```

### Anatomy of `declarch.kdl`

**KDL Syntax** — clean, flexible, and readable.

```kdl
// ~/.config/declarch/declarch.kdl

// Set your editor
editor "nvim"

// AUR packages (Arch Linux only)
packages:aur {
    hyprland
    waybar
}

// Cross-distro static binaries (works everywhere)
packages {
    bat
    exa
    ripgrep
}

// Flatpak apps (works on any Linux)
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}

// Import other modules
imports {
    modules/gaming
    modules/development
}
```

**Alternative: Embedded Syntax**

```kdl
packages {
    // Default to Soar
    bat
    exa

    // Nested backend-specific
    aur {
        hyprland
    }

    flatpak {
        com.spotify.Client
    }
}
```

**NEW: Inline Prefix Syntax** (Most Flexible)

```kdl
packages {
    // Mix different backends in one block
    hyprland           // Default (AUR)
    aur:waybar         // Explicit AUR
    soar:bat           // Soar (also works with "app:bat")
    soar:exa           // Soar
    flatpak:com.spotify.Client

    // Can also combine with nested blocks
    aur {
        swww
    }
}
```

**Advanced KDL Syntax**

declarch supports powerful configuration features beyond package declarations:

```kdl
// === META INFORMATION ===
meta {
    description "My Hyprland Workstation"
    author "nixval"
    version "1.0.0"
    tags "workstation" "hyprland" "development"
    url "https://github.com/nixval/dotfiles"
}

// === CONFLICTS ===
// Define mutually exclusive packages
conflicts {
    vim neovim           // Can't have both installed
    pipewire pulseaudio  // Audio system choice
}

// === BACKEND OPTIONS ===
// Configure package manager behavior
options:aur {
    noconfirm            // Skip confirmation prompts
    helper "paru"        // Use paru instead of yay
}

// === ENVIRONMENT VARIABLES ===
// Set environment variables for package operations
env EDITOR="nvim" VISUAL="nvim"

// Backend-specific environment variables
env:aur MAKEFLAGS="-j4"  // Parallel AUR builds

// === REPOSITORIES ===
// Add custom package repositories
repos:aur {
    "https://aur.archlinux.org"
}

repos:flatpak {
    "https://flathub.org/repo/flathub.flatpakrepo"
}

// === POLICY CONTROL ===
// Define package lifecycle policies
policy {
    protected {
        linux        // Never remove these packages
        systemd
        base-devel
    }
    orphans "keep"   // Strategy: "keep", "remove", or "ask"
}

// === HOOKS ===
// Run commands before/after sync
//
// Flat syntax (recommended):
on-pre-sync "notify-send 'Starting sync...'"
on-sync "notify-send 'Packages updated'"
on-sync-sudo "systemctl restart gdm"
//
// Or nested syntax (still supported):
// hooks {
//     pre-sync {
//         run "notify-send 'Starting package sync...'"
//     }
//     post-sync {
//         run "notify-send 'Packages updated'"
//         sudo-needed "systemctl restart gdm"
//     }
// }
```

**⚠️ Hooks Security:**

Hooks are disabled by default for security. Enable with `--hooks` flag:

```bash
# Preview hooks (always safe - shows what would run)
declarch sync --dry-run

# Execute hooks (after reviewing config)
declarch sync --hooks
```

**Why?** Remote configs may contain arbitrary commands. Always review before enabling.

**Module Configurations**

All advanced syntax features work in module files too! Modules can define their own meta, conflicts, env, etc., which are merged with the root config:

```kdl
// ~/.config/declarch/modules/development.kdl

meta {
    description "Development tools and IDEs"
    author "nixval"
    tags "development"
}

env EDITOR="nvim"

packages {
    aur:neovim
    soar:ripgrep
}

policy {
    protected {
        neovim
    }
}

// Flat syntax for hooks
on-sync "notify-send 'Dev tools updated'"
```

**Module Merging Behavior:**
- **Meta**: First config wins (usually from root)
- **Conflicts**: Accumulated from all configs
- **Backend Options**: Later configs override earlier ones
- **Environment Variables**: Later configs extend earlier ones
- **Repositories**: Later configs extend earlier ones
- **Policy**: Last one wins
- **Hooks**: Later configs extend earlier ones

-----

## 🛠️ Usage

### The Magic Command

Sync your system to match your config:

```bash
declarch sync
```

With system update and pruning:

```bash
declarch sync -u --prune
```

### Common Commands

| Command | Description |
| :--- | :--- |
| `declarch init` | Create or fetch configuration. |
| `declarch edit` | Edit config in your editor. |
| `declarch check` | Validate syntax and show packages. |
| `declarch info` | Show system status and managed packages. |
| `declarch sync` | Install/remove packages to match config. |
| `declarch switch` | Replace one package with another. |

### Useful Flags

| Flag | Description |
| :--- | :--- |
| `-u` / `--update` | Run `paru -Syu` before syncing. |
| `--dry-run` | Preview changes without executing. |
| `--prune` | Remove managed packages not in config. |
| `--target <NAME>` | Sync only specific package or module. |
| `--noconfirm` | Skip package manager prompts (CI/CD). |

### Edit Configuration

Opens your config file in your editor:

```bash
declarch edit
```

Editor priority (first found wins):
1. `editor "nvim"` in your `declarch.kdl`
2. `$EDITOR` environment variable
3. `$VISUAL` environment variable
4. `nano` (default fallback)

-----

## 🌍 Remote Init

**Go-style package importing for configs.**

Fetch configurations from any Git repository without PRing to a central registry.

### Examples

```bash
# User's GitHub repository
declarch init myuser/hyprland-setup

# Official project config
declarch init hyprwm/hyprland

# Community registry
declarch init gaming/steam-setup

# Config variant (multiple configs in one repo)
declarch init myuser/dotfiles:uwsm

# Specific branch
declarch init myuser/dotfiles/develop:uwsm
```

### How It Works

1. **GitHub**: `user/repo` → fetches `declarch.kdl` from repository root
2. **GitLab**: `gitlab.com/user/repo` → fetches from GitLab
3. **Direct URL**: Full URL to any `.kdl` file
4. **Community**: Fetch from official [declarch-packages](https://github.com/nixval/declarch-packages) registry

**Repository Requirements:**
- File named `declarch.kdl` at repository root
- Valid KDL syntax
- Public access (or authentication)

See [Remote Init Guide](https://github.com/nixval/declarch/wiki/Remote-Init-Guide) for details.

-----

## 📦 Package Backends

### AUR (Arch User Repository)

Arch Linux packages from the community.

```kdl
packages:aur {
    hyprland
    waybar
}
```

**Requires:** AUR helper (`paru` or `yay`)

---

### Flatpak

Universal packages — works on any Linux distribution.

```kdl
packages:flatpak {
    com.spotify.Client
    org.mozilla.firefox
}
```

**Requires:** `flatpak` installed

---

### Soar (Cross-Distro Static Binaries)

Pre-built static binaries that work on any Linux.

```kdl
packages {
    bat
    exa
    ripgrep
    fd
}
```

**Requires:** Nothing (auto-installs Soar if needed)

-----

## 💡 Why KDL?

We chose [KDL](https://kdl.dev/) because it's designed for configuration, not data serialization.

  * **VS JSON:** Comments are supported! `// like this`
  * **VS YAML:** No whitespace/indentation anxiety
  * **VS TOML:** Better support for nested hierarchies (blocks)
  * **Human-Readable:** Clean, minimal syntax

-----

## ⚠️ Safety First

`declarch` keeps its state in `~/.local/state/declarch/state.json`.

  * If you delete a package from your `.kdl` file, `declarch` will **NOT** remove it from your system unless you run `sync --prune`.
  * It automatically creates a backup of the state file before every write operation.

-----

## 📚 Documentation

- [Home](https://github.com/nixval/declarch/wiki/Home) - Overview
- [Installation Guide](https://github.com/nixval/declarch/wiki/Installation) - Detailed installation
- [Quick Start](https://github.com/nixval/declarch/wiki/Quick-Start) - First steps
- [KDL Syntax Reference](https://github.com/nixval/declarch/wiki/KDL-Syntax-Reference) - Complete syntax
- [Remote Init Guide](https://github.com/nixval/declarch/wiki/Remote-Init-Guide) - Fetch from GitHub/GitLab
- [Examples](https://github.com/nixval/declarch/wiki/Examples) - Real-world configs
- [Backend System](docs/Backend-System.md) - Generic backend architecture
- [Testing Guide](TESTING.md) - Test the new backend system
- [Quick Test](QUICK-TEST.md) - Ready-to-use test configs

-----

## 🤝 Contributing

Pull requests are welcome! This project is written in **Rust**.

1. Fork it
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

-----

## 📜 License

**MIT**

-----

