# Auto-Installation Guide

## Overview

declarch automatically installs **Soar** during initialization, making it truly portable across all Linux distributions without manual setup.

## Installation Methods

### Method 1: Install Script (Recommended)

```bash
# Via curl
curl -fsSL https://raw.githubusercontent.com/user/declarch/main/install.sh | sh

# Via wget
wget -qO- https://raw.githubusercontent.com/user/declarch/main/install.sh | sh

# From source locally
git clone https://github.com/user/declarch
cd declarch
./install.sh --local
```

**What happens automatically:**
1. ✅ Installs declarch binary
2. ✅ Downloads and installs Soar (if not found)
3. ✅ Checks for AUR helpers (paru/yay)
4. ✅ Checks for Flatpak
5. ✅ Ready to use immediately!

### Method 2: Cargo Install

```bash
cargo install declarch
```

Then run init to auto-install Soar:
```bash
declarch init
```

**What happens:**
- Checks if Soar is installed
- If not found, installs Soar automatically
- Creates configuration files
- Initializes state

### Method 3: From Source

```bash
git clone https://github.com/user/declarch
cd declarch
cargo build --release
cargo install --path .
declarch init  # Auto-installs Soar
```

## User Experience

### First-Time User (Arch Linux)

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/user/declarch/main/install.sh | sh

# Output:
# 🚀 Installing declarch...
# 📦 Installing declarch via cargo...
# ✅ declarch installed successfully
# 📦 Installing Soar (cross-distro package manager)...
# ✅ Soar installed successfully
#
# 🔍 Checking for additional package managers...
#   ✅ paru found (AUR support enabled)
#   ✅ flatpak found (Flatpak support enabled)
#
# ✨ Installation complete!

# Initialize
declarch init

# Output:
# Initializing declarch root
# ✅ Soar is installed and ready
# ✅ Created config directory: ~/.config/declarch
# ✅ Created config file: ~/.config/declarch/packages.kdl
# ✅ Initialized state for host: myhost

# Use immediately
declarch sync
# Works! All backends ready
```

### First-Time User (Debian/Ubuntu)

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/user/declarch/main/install.sh | sh

# Output:
# 🚀 Installing declarch...
# 📦 Installing declarch via cargo...
# ✅ declarch installed successfully
# 📦 Installing Soar (cross-distro package manager)...
# ✅ Soar installed successfully
#
# 🔍 Checking for additional package managers...
#   ⚠️  No AUR helper found. (Not on Arch, expected)
#   ⚠️  flatpak not found. Install for Flatpak support:
#      sudo apt install flatpak
#
# ✨ Installation complete!

# Initialize
declarch init

# Create config
cat > ~/.config/declarch/packages.kdl << 'EOF'
packages {
    bat
    exa
    ripgrep
}
EOF

# Sync - Works on Debian!
declarch sync
# Installs bat, exa, ripgrep via Soar automatically
```

## Configuration Examples

### Cross-Distro Config (Works Everywhere)

```kdl
// ~/.config/declarch/packages.kdl

// Soar packages - works on ALL distros
packages {
    bat
    exa
    fd
    ripgrep
    fzf
    zoxide
}

// AUR packages - Arch only (auto-skipped on Debian/Ubuntu)
aur-packages {
    hyprland
    waybar
    swww
}

// Flatpak packages - cross-distro
flatpak-packages {
    com.spotify.Client
    org.mozilla.firefox
}
```

**Behavior by distro:**

| Package | Arch | Debian/Ubuntu | Fedora |
|---------|------|---------------|--------|
| `packages {}` | Soar ✅ | Soar ✅ | Soar ✅ |
| `aur-packages {}` | AUR ✅ | Skipped ⏭️ | Skipped ⏭️ |
| `flatpak-packages {}` | Flatpak ✅ | Flatpak ✅ | Flatpak ✅ |

## Advanced Options

### Skip Auto-Install (Advanced Users)

If you want to manage Soar installation manually:

```bash
# Skip during init
declarch init --skip-soar-install

# Skip during sync
declarch sync --skip-soar-install
```

### Manual Soar Installation

If auto-install fails:

```bash
curl -fsSL https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh

# Or with wget
wget -qO- https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh
```

Then verify:
```bash
which soar
soar --version
```

## How It Works

### Installation Flow

```
install.sh
    ├─> Install declarch (cargo/git/local)
    ├─> Check Soar
    │   ├─> Found? → Show version
    │   └─> Missing? → Auto-install
    │       ├─> Try curl
    │       └─> Fallback wget
    ├─> Check AUR helpers (paru/yay)
    ├─> Check Flatpak
    └─> Done!
```

### Init Flow

```
declarch init
    ├─> Check Soar
    │   ├─> Installed? → Continue
    │   └─> Missing? → Auto-install
    │       ├─> Success? → Continue
    │       └─> Failed? → Warn, continue without Soar
    ├─> Create config dir
    ├─> Create config file
    └─> Initialize state
```

### Sync Flow (Lazy Install)

```
declarch sync
    ├─> Load config
    ├─> Detect backends needed
    │   └─> Soar required?
    │       ├─> Yes + Not installed → Auto-install
    │       │   ├─> Success? → Use Soar
    │       │   └─> Failed? → Skip Soar packages
    │       └─> No or --skip-soar-install → Skip
    ├─> Sync packages
    └─> Done!
```

## Troubleshooting

### Soar Installation Failed

**Symptom:**
```
❌ Soar installation failed
```

**Solutions:**

1. **Install manually:**
   ```bash
   curl -fsSL https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh
   ```

2. **Check curl/wget:**
   ```bash
   which curl wget
   sudo apt install curl  # Debian/Ubuntu
   sudo pacman -S curl   # Arch
   ```

3. **Check internet connection:**
   ```bash
   curl -I https://github.com
   ```

### Soar Not Found After Init

**Symptom:**
```
Soar is required but not installed
```

**Solution:**
```bash
# Install manually
curl -fsSL https://raw.githubusercontent.com/pkgforge/soar/main/install.sh | sh

# Verify
which soar
```

### Want to Disable Auto-Install Permanently

**Option 1: Alias**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias declarch='declarch --skip-soar-install'
```

**Option 2: Environment Variable** (Future feature)
```bash
export DECLARCH_SKIP_SOAR=1
```

## Best Practices

### 1. Let Auto-Install Handle It
Recommended for most users:
```bash
declarch init  # Auto-installs Soar
```

### 2. Verify Installation
```bash
declarch info  # Shows all backends
```

### 3. Test Configuration
```bash
declarch check --verbose  # See what will be installed
```

### 4. Dry Run First
```bash
declarch sync --dry-run  # Preview changes
```

## FAQ

**Q: Can I use declarch without Soar?**
A: Yes on Arch Linux (AUR + Flatpak). On other distros, Soar is required for core functionality.

**Q: Is Soar safe?**
A: Yes. Soar is a well-maintained package manager for static binaries. It installs to `~/.local/share/soar` by default, isolated from system packages.

**Q: Can I uninstall Soar?**
A: Yes, but declarch won't be able to install `packages {}` on non-Arch distros.

**Q: Does declarch install AUR helpers?**
A: No. Paru/yay/flatpak must be installed manually (as they should be).

**Q: What if I don't have curl or wget?**
A: Install curl first:
```bash
# Debian/Ubuntu
sudo apt install curl

# Arch
sudo pacman -S curl

# Fedora
sudo dnf install curl
```

**Q: Can I use a different Soar installation method?**
A: Yes. Install Soar however you prefer, declarch just needs it in PATH.

## Summary

- ✅ **Zero-config on Arch**: Works immediately if you have paru/yay
- ✅ **One-command on Debian/Ubuntu**: Just run install script
- ✅ **Seamless**: Soar installed automatically, no manual steps
- ✅ **Skippable**: Use `--skip-soar-install` if you prefer manual setup
- ✅ **Safe**: Only Soar is auto-installed, not AUR helpers or Flatpak
- ✅ **Portable**: Same configuration works on all distros

Enjoy cross-distro package management! 🚀
