# 🌱 **Declarch**

<p align="center">
<strong>Universal declarative package manager — powered by Rust.</strong><br>
Unify AUR, flatpak, npm, cargo, pip, and custom backends under one declarative config
</p>


<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Build" src="https://img.shields.io/badge/status-v0.4.2-orange">
  <img alt="Arch" src="https://img.shields.io/badge/arch-linux-blue">
  <img alt="Rust" src="https://img.shields.io/badge/built_with-rust-orange">
</p>

-----

⚠️ Important Warning (Read This First)

declarch is still in BETA.
- Architecture is fragile by design — this tool is a wrapper on top of wrappers (paru/yay, flatpak, npm, pip, etc).
- Only tested on Arch-based distros (Arch Linux & EndeavourOS).
- Other backends (Flatpak, npm, bun, pip, cargo, soar) may work on other distros, but you’re on your own.
- You must install each backend manually (declarch does NOT bootstrap package managers).
- Expect breaking changes, rough edges, and footguns.

👉 If you want absolute stability, this is not that tool (yet).
👉 If you like declarative systems and accept risk, welcome.

## 🧠 Why declarch Exists

Linux package management is mostly imperative.

You run commands like:
```bash
paru -S something
npm install -g random-tool
pip install whatever
```
A month later:
- You forgot why it was installed
- You don’t know where it came from
- Your system becomes a package graveyard
declarch flips that.

You declare what you want, not what you ran.

```kdl
// ~/.config/declarch/declarch.kdl

// AUR packages (Default from aur)
packages {
    hyprland
    waybar
}

// Static binaries (Soar)
packages:soar {
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

Share your config across others

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

---

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

# From GitLab
declarch init gitlab.com/user/repo

# Direct URL
declarch init https://example.com/config.kdl

# Declarch-Packages repositories
declarch init wm/hyprland
```

### Anatomy of `declarch.kdl`

**KDL Syntax** — clean, flexible, and readable.

```kdl
// ~/.config/declarch/declarch.kdl

// Set your editor (default nano)
// use declarch edit 
editor "nvim"

// Import other modules
imports {
    modules/base // use declarch edit base
    modules/gaming  // use declarch edit gaming
    modules/development 
    modules/hyprland/mydotfiles // use declarch edit hyprland/mydotfiles
}


packages {
	// AUR packages as default
    hyprland
    waybar
    
    // Horizontal writing
    rofi kitty flatpak:org.mozzilla.firefox wl-clipboard
    
    // Embedded Syntax
    soar{
        bat
	    exa
    }
    // inline prefix
    flatpak:com.spotify.Client
    npm:npmstat
    pip:furl
    cargo:tealdeer
}

packages:soar {
	ripgrep
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
d
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


## ⚠️ Safety First

`declarch` keeps its state in `~/.local/state/declarch/state.json`.

  * If you delete a package from your `.kdl` file, `declarch` will **NOT** remove it from your system unless you run `sync --prune`.
  * It automatically creates a backup of the state file before every write operation.

-----

## ⚠️ Package Name Conflicts

Declarch tracks packages separately for each backend, which means you can have the same package name installed from different backends:

```kdl
packages {
    claude-cli       // AUR (default)
    npm:claude-cli   // npm
    bun:claude-cli   // Bun
}
```

**This works**, but be aware:
- Each backend installs to different locations
- AUR → `/usr/bin/claude-cli`
- npm → `~/.npm-global/bin/claude-cli`
- bun → `~/.local/bin/claude-cli`
- **Your PATH ordering determines which one runs!**

### Checking for Conflicts

Use `--conflicts` flag to detect potential conflicts:

```bash
declarch check --conflicts
```

Example output:
```
⚠ Found 1 package name conflicts across backends:

These packages have the same name but different backends:
They will be installed separately by each backend.
Watch out for PATH conflicts!

  ⚠️  claude-cli
     └─ npm
     └─ bun
     └─ aur
```

Use `declarch info` to see which backends have installed which packages.

## 🤝 Contributing

Pull requests are welcome! I only test this out in arch and endeavour. It possible to add nala, dnf5, or anything directly. For now it also available in backend.kdl, but it possible to be inside this tools if stable and tested out. This project is written in **Rust**.

1. Fork it
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

-----

## 📜 License

**MIT**

-----

