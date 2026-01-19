# Configuration Examples

Real-world declarch configuration examples for various use cases.

---

## Table of Contents

- [Minimal Setup](#minimal-setup)
- [Desktop Environment](#desktop-environment)
- [Development Environment](#development-environment)
- [Gaming Setup](#gaming-setup)
- [Modular Setup](#modular-setup)
- [Host-Specific Configs](#host-specific-configs)
- [Cross-Distro Setup](#cross-distro-setup)

---

## Minimal Setup

**For:** Users who want a simple, lightweight system.

**`declarch.kdl`**
```kdl
// Minimal Arch Setup
// Last updated: 2025-01-13

// Essential CLI tools
packages {
    bat       // Better cat
    exa       // Better ls
    fd        // Better find
    ripgrep   // Fast grep
    fzf       // Fuzzy finder
}

// Editor
packages:aur {
    neovim
}

// Set default editor
editor "nvim"
```

---

## Desktop Environment

**For:** Full Hyprland desktop setup with all essentials.

**`declarch.kdl`**
```kdl
// My Hyprland Desktop
// Author: Your Name
// Source: https://github.com/username/dotfiles

editor "nvim"

description {
    Complete Hyprland desktop with Waybar, SDDM, and gaming support.
    Focused on minimalism and performance.
}

// --- Window Manager ---
packages:aur {
    hyprland
    waybar
    swww
    wlogout
    wofi
    hyprpaper
    hyprpicker
    xdg-desktop-portal-hyprland
}

// --- Terminal & Shell ---
packages:aur {
    alacritty
    zsh
    oh-my-zsh-git
}

packages {
    starship   # Cross-shell prompt
}

// --- Launcher & Menu ---
packages:aur {
    nwg-dock-hyprland
    nwg-menu
}

// --- Authentication ---
packages:aur {
    polkit-kde-agent
}

// --- System Utilities ---
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    zoxide
    btop
    neofetch
}

// --- File Management ---
packages:aur {
    thunar
    thunar-archive-plugin
}

packages:flatpak {
    com.github.xournalp.xournalpp  # Note-taking
}

// --- Screenshot & Screen Recording ---
packages:aur {
    grim
    slurp
    swappy
    wf-recorder
    wl-clipboard
}

// --- PDF & Documents ---
packages:aur {
    zathura
    zathura-pdf-mupdf
}

// --- Gaming ---
imports {
    modules/gaming
}

// --- Communication ---
imports {
    modules/communication
}
```

**`modules/gaming.kdl`**
```kdl
// Gaming Setup
// Steam, Lutris, game launchers

description "PC gaming with Steam and Lutris"

// Steam
packages:aur {
    steam
}

packages:flatpak {
    com.valvesoftware.Steam
    com.valvesoftware.Steam.CompatibilityTool.Proton-GE
}

// Game launchers
packages:flatpak {
    net.lutris.Lutris
    com.heroicgameslauncher.hgl
    org.prismlauncher.PrismLauncher
}

// Gaming tools
packages:aur {
    mangohud
    gamemode
    goverlay
}

// Discord
packages:flatpak {
    com.discordapp.Discord
}
```

**`modules/communication.kdl`**
```kdl
// Communication Apps
// Email, chat, video calls

description "Email, messaging, and video conferencing"

// Email
packages:flatpak {
    org.gnome.Evolution
}

// Messaging
packages:flatpak {
    org.telegram.desktop
    com.discordapp.Discord
    com.slack.Slack
}

// Video calls
packages:flatpak {
    com.spotify.Client  # Music
    us.zoom.Zoom        # Video calls
}
```

---

## Development Environment

**For:** Developers working with multiple languages.

**`declarch.kdl`**
```kdl
// Development Environment
// Cross-language development setup

editor "nvim"

description {
    Complete development environment for:
    - Rust (systems programming)
    - Node.js (web development)
    - Python (data science, scripting)
    - Go (backend services)
}

// --- Version Control ---
packages {
    git
    git-delta  # Better diff viewer
    gh         # GitHub CLI
    lazygit    # Git TUI
}

// --- Editors ---
packages:aur {
    neovim
    vim
}

packages:flatpak {
    com.visualstudio.code  # VS Code
}

// --- Rust Development ---
packages {
    rustup
}

// Rust tools installed via cargo
// (not managed by declarch)
// cargo install ripgrep fd-find bat git-delta

// --- Node.js Development ---
packages {
    nodejs
    npm
}

packages:aur {
    nvm  # Node version manager
}

// Common npm packages (global)
// npm install -g typescript prettier eslint

// --- Python Development ---
packages {
    python
    python-pip
}

packages:aur {
    python-poetry
    ruff
    black
}

// --- Go Development ---
packages {
    go
}

// --- Docker ---
packages {
    docker
    docker-compose
}

// --- Database Tools ---
packages:aur {
    dbeaver
}

// --- API Testing ---
packages:flatpak {
    com.postman.Postman
}

// --- CLI Development Tools ---
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    jq        # JSON processor
    httpie    # HTTP client
}

// --- Build Tools ---
packages {
    base-devel  # Arch build tools
    cmake
    make
    gcc
}

// --- Language Servers ---
// (For Neovim/Vim LSP)
packages:aur {
    rust-analyzer
    gopls
    python-lsp-server
    bash-language-server
}
```

---

## Gaming Setup

**For:** Gaming-focused system with Steam, Lutris, emulators.

**`declarch.kdl`**
```kdl
// Ultimate Gaming Setup
// Focus on compatibility and performance

description {
    Arch Linux gaming machine with:
    - Steam (Proton, Wine)
    - Lutris (Game launchers)
    - Emulators (RetroArch, Yuzu)
    - Performance tools (MangoHud, GameMode)
}

// --- Graphics Drivers ---
// Choose based on your GPU:
// NVIDIA:
packages:aur {
    nvidia
    nvidia-utils
}
// AMD:
// packages:aur {
//     xf86-video-amdgpu
//     vulkan-radeon
// }

// --- Steam ---
packages:aur {
    steam
}

packages:flatpak {
    com.valvesoftware.Steam
    com.valvesoftware.Steam.CompatibilityTool.Proton-GE
}

// --- Game Launchers ---
packages:flatpak {
    net.lutris.Lutris
    com.heroicgameslauncher.hgl
    org.prismlauncher.PrismLauncher
}

// --- Emulators ---
packages:flatpak {
    org.libretro.RetroArch
    io.github.yuzu_emu.yuzu
}

// --- Gaming Tools ---
packages:aur {
    mangohud      # FPS overlay
    gamemode      # Performance optimization
    goverlay      # MangoHud GUI
    gamescope     # Compositor for games
    vkbasalt      # Post-processing
}

// --- Discord & Streaming ---
packages:flatpak {
    com.discordapp.Discord
}

// --- Controller Support ---
packages:aur {
    xboxdrv
    ds4drv
}

// --- Screen Recording ---
packages:aur {
    obs-studio
}

// --- System Optimization ---
packages {
    zram-generator
}

packages:aur {
    irqbalance  # CPU interrupt balancing
}
```

---

## Modular Setup

**For:** Organizing config into multiple files.

**`~/.config/declarch/declarch.kdl`** (Root)
```kdl
// My Dotfiles
// Modular setup with separate configs

// Global settings
editor "nvim"

// Base packages for all hosts
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    zoxide
    btop
}

// Import modules
imports {
    modules/window-manager
    modules/terminal
    modules/development
    modules/gaming
    modules/communication
}
```

**`~/.config/declarch/modules/window-manager.kdl`**
```kdl
// Window Manager Setup
// Hyprland configuration

packages:aur {
    hyprland
    waybar
    swww
    wlogout
    wofi
    hyprpaper
}
```

**`~/.config/declarch/modules/terminal.kdl`**
```kdl
// Terminal & Shell
// Alacritty + Zsh + Starship

packages:aur {
    alacritty
    zsh
}

packages {
    starship
}
```

**`~/.config/declarch/modules/development.kdl`**
```kdl
// Development Tools
// Git, editors, LSPs

packages {
    git
    gh
}

packages:aur {
    neovim
    git-delta
    lazygit
}
```

---

## Host-Specific Configs

**For:** Different packages for desktop vs laptop.

**`~/.config/declarch/declarch.kdl`** (Root)
```kdl
// Multi-Host Setup
// Base packages + host-specific

// Base packages (all hosts)
packages {
    bat
    exa
    fd
    ripgrep
}

// Host-specific modules
imports {
    hosts/desktop  # Only on desktop
    hosts/laptop   # Only on laptop
}
```

**`~/.config/declarch/hosts/desktop.kdl`**
```kdl
// Desktop-Specific Packages
// Gaming, development, multiple monitors

description "Desktop workstation with gaming setup"

// GPU-intensive apps
packages:aur {
    steam
    blender
    krita
}

packages:flatpak {
    com.valvesoftware.Steam
    org.blender.Blender
}

// Multiple monitor support
packages:aur {
    autorandr
}
```

**`~/.config/declarch/hosts/laptop.kdl`**
```kdl
// Laptop-Specific Packages
// Battery optimization, mobile tools

description "Laptop setup with battery optimization"

// Power management
packages:aur {
    tlp
    thermald
    auto-cpufreq
}

// Mobile productivity
packages:flatpak {
    com.jgraph.drawio  # Diagrams
    org.libreoffice.LibreOffice  # Documents
}

// Bluetooth
packages:aur {
    bluez
    bluez-utils
}
```

---

## Cross-Distro Setup

**For:** Config that works on Arch, Fedora, Ubuntu.

**`declarch.kdl`**
```kdl
// Cross-Distro Setup
// Works on Arch, Fedora, Ubuntu, etc.

description {
    Portable configuration that works on any Linux distribution.
    Uses only Soar (static binaries) and Flatpak (universal packages).
}

// --- Soar Packages (Cross-Distro) ---
packages {
    // CLI tools (work everywhere)
    bat
    exa
    fd
    ripgrep
    fzf
    zoxide

    // Development tools
    gh
    lazygit

    // System utilities
    btop
    neofetch
}

// --- Flatpak Packages (Cross-Distro) ---
packages:flatpak {
    // Browsers
    org.mozilla.firefox
    com.google.Chrome

    // Communication
    org.telegram.desktop
    com.discordapp.Discord

    // Productivity
    com.spotify.Client
    org.libreoffice.LibreOffice
    com.jgraph.drawio

    // Development
    com.visualstudio.code
    com.postman.Postman

    // Utilities
    com.github.maasjan.ripper
}

// NOTE: No AUR packages (Arch-specific)
// This config will work on Fedora, Ubuntu, etc.
```

---

## Server Setup

**For:** Headless server, minimal GUI.

**`declarch.kdl`**
```kdl
// Server Configuration
// Minimal packages for headless server

description "Minimal server setup"

// Essential CLI tools
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    htop
    btop
    tmux
}

// Development tools
packages {
    git
    gh
    neovim
}

// Monitoring
packages {
    netdata
}

// Network tools
packages {
    curl
    wget
    rsync
    sshfs
}

// NOTE: No GUI packages, no gaming, no multimedia
```

---

## Audio Production Setup

**For:** Music production, recording, mixing.

**`declarch.kdl`**
```kdl
// Audio Production Setup
// Music production on Linux

description {
    Audio production workstation with:
    - DAW ( Ardour, Bitwig)
    - Plugins (LV2, VST)
    - Audio tools (Audacity, Reaper)
}

// --- DAW (Digital Audio Workstation) ---
packages:aur {
    ardour
}

packages:flatpak {
    com.bitwig.BitwigStudio
}

// --- Audio Tools ---
packages:aur {
    audacity
    helm
}

packages:flatpak {
    com.reaper.Reaper
}

// --- Plugins ---
packages:aur {
    lsp-plugins
    vital
}

// --- Audio System ---
packages:aur {
    pipewire
    pipewire-jack
    qpwgraph
}

// --- MIDI Tools ---
packages:aur {
    qmidinet
}
```

---

## Security & Privacy Setup

**For:** Security-focused system with encryption tools.

**`declarch.kdl`**
```kdl
// Security & Privacy Setup
// Encryption, anonymity, secure communication

description "Privacy-focused Linux setup"

// --- Encryption ---
packages {
    cryptsetup
    veracrypt
}

packages:aur {
    tomb
}

// --- Password Management ---
packages:flatpak {
    com.bitwarden.desktop
}

// --- Secure Communication ---
packages:flatpak {
    org.signal.Signal
    im.riot.Riot  # Element (Matrix)
}

// --- Privacy Tools ---
packages:flatpak {
    com.github.muflone.privacyreminder
    org.torproject.torbrowser-launcher
}

// --- Firewall ---
packages {
    firewalld
}

// --- Network Monitoring ---
packages {
    wireshark
    nmap
}

// --- File Encryption ---
packages:aur {
    age
    go-yq
}
```

---

## Automation with Hooks

**For:** Running commands before/after sync operations.

### Basic Notifications

```kdl
// Notify when packages are updated
on-sync "notify-send 'Declarch' 'Packages synchronized successfully'"
```

### Backup Before Sync

```kdl
// Backup package list before sync
on-pre-sync "pacman -Qqe > ~/.config/pacman/pkglist.before.txt"

// Update state after sync
on-sync "pacman -Qqe > ~/.config/pacman/pkglist.txt"
```

### Restart Services

```kdl
// Restart display manager after GPU driver updates
on-sync-sudo "systemctl restart gdm"

// Restart PipeWire after audio updates
on-sync-sudo "systemctl --user restart wireplumber pipewire pipewire-pulse"
```

### Clear Caches

```kdl
// Clean package cache after sync
on-sync "paccache -rk3"  // Keep last 3 versions

// Clean thumbnail cache
on-sync "rm -rf ~/.cache/thumbnails/*"
```

### Development Environment Hooks

```kdl
// Update Rust after sync
on-sync "rustup update"

// Update Python packages
on-sync "pipx upgrade-all"

// Update global npm packages
on-sync "npm update -g"
```

### Multiple Hooks

```kdl
// Pre-sync: Backup and notify
on-pre-sync "echo '[$(date)] Starting sync...' >> ~/.declarch.log"
on-pre-sync "notify-send 'Declarch' 'Starting package synchronization'"

// Post-sync: Notify and clean up
on-sync "notify-send 'Declarch' 'Packages updated successfully'"
on-sync "rm -rf ~/.cache/thumbnails/*"

// Post-sync with sudo: Restart services
on-sync-sudo "systemctl restart NetworkManager"
```

### Hook Security

**Important:** Hooks are disabled by default for security. Enable with `--hooks`:

```bash
# Preview hooks (always safe)
declarch sync --dry-run

# Execute hooks (after reviewing config)
declarch sync --hooks
```

**Why?**
- Remote configs may contain arbitrary commands
- Always review configs before enabling hooks
- Dry-run shows what will be executed

---

## Tips for Your Own Config

### 1. Start Simple

Begin with essential packages, add more as needed.

### 2. Use Descriptions

```kdl
description "My personal setup for web development"
```

Helps you remember what the config is for.

### 3. Organize by Purpose

Group related packages:
```kdl
// CLI tools
packages {
    bat
    exa
}

// Development
packages:aur {
    neovim
}
```

### 4. Use Aliases for Clarity

```kdl
aliases-pkg {
    browser firefox
}

packages:aur {
    browser  # Installs firefox
}
```

### 5. Version Control Your Config

```bash
cd ~/.config/declarch
git init
git add declarch.kdl modules/
git commit -m "Initial config"
```

---

## Need More Examples?

- **KDL Syntax**: [KDL Syntax Reference](KDL-Syntax-Reference.md)
- **Remote Init**: [Remote Init Guide](Remote-Init-Guide.md)
- **Community Configs**: https://github.com/nixval/declarch-packages

---

**Have a great config example? Share it! 🚀**
