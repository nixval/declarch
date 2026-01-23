# Installation Guide

Complete guide to installing declarch on Arch Linux and other distributions.

---

## Table of Contents

- [Requirements](#requirements)
- [Installation Methods](#installation-methods)
- [Post-Installation](#post-installation)
- [Troubleshooting](#troubleshooting)

---

## Requirements

### Arch Linux

**Required:**
- Arch Linux or Arch-based distribution (EndeavourOS, Manjaro, Garuda, etc.)
- An AUR helper: `paru` or `yay`
- `git` (for remote init)

**Optional:**
- `flatpak` (for Flatpak backend)
- `soar` (auto-installed if needed)

### Other Distributions

Declarch can work on other Linux distributions with limited functionality:

- ✅ **Soar packages** - Static binaries, works everywhere
- ✅ **Flatpak packages** - Works if Flatpak is installed
- ❌ **AUR packages** - Arch-specific, not available

---

## Installation Methods

### Method 1: AUR (Recommended for Arch)

#### Using paru

```bash
paru -S declarch
```

#### Using yay

```bash
yay -S declarch
```

**What gets installed:**
- `declarch` binary → `/usr/bin/declarch`
- Config directory → `~/.config/declarch/`
- Auto-completion for bash, zsh, fish

---

### Method 2: Pre-built Binary

```bash
# Download latest release
wget https://github.com/nixval/declarch/releases/latest/download/declarch-x86_64.tar.gz

# Extract
tar -xvf declarch-x86_64.tar.gz

# Install
sudo install declarch /usr/local/bin/

# Verify
declarch --version
```

---

### Method 3: Install Script

**Automated install (recommended for quick setup):**

```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

**What the script does:**
1. Detects your AUR helper (paru/yay)
2. Installs declarch from AUR
3. Sets up shell completions
4. Verifies installation

---

### Method 4: Build from Source

#### Prerequisites

```bash
# Install Rust toolchain
sudo pacman -S rust git

# Or install rustup instead
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Build and Install

```bash
# Clone repository
git clone https://github.com/nixval/declarch.git
cd declarch

# Build release binary
cargo build --release

# Install
sudo install target/release/declarch /usr/local/bin/

# Verify
declarch --version
```

---

### Method 5: Cargo (For Developers)

```bash
cargo install declarch
```

**Note:** This installs to `~/.cargo/bin/`, make sure it's in your PATH.

---

## Post-Installation

### 1. Verify Installation

```bash
declarch --version
# Output: declarch 0.4.0

declarch --help
# Shows all available commands
```

---

### 2. Initialize Configuration

```bash
# Create empty config
declarch init

# Or fetch from a repository
declarch init hyprland/niri-nico
```

This creates:
- `~/.config/declarch/declarch.kdl` - Main configuration
- `~/.config/declarch/modules/` - Module directory

---

### 3. (Optional) Set Default Editor

Add to your `declarch.kdl`:

```kdl
editor "nvim"  # or vim, code, micro, etc.
```

Or set environment variable in `~/.bashrc` or `~/.zshrc`:

```bash
export EDITOR=nvim
export VISUAL=nvim
```

---

### 4. (Optional) Enable Shell Completions

#### Bash

```bash
# Add to ~/.bashrc
source <(declarch completions bash)
```

#### Zsh

```bash
# Add to ~/.zshrc
source <(declarch completions zsh)
```

#### Fish

```bash
# Add to ~/.config/fish/config.fish
declarch completions fish | source
```

---

## Installation by Distribution

### Arch Linux

```bash
paru -S declarch
```

✅ Full functionality (AUR + Soar + Flatpak)

---

### EndeavourOS

```bash
# Already has yay installed
yay -S declarch
```

✅ Full functionality

---

### Manjaro

```bash
yay -S declarch
# or
paru -S declarch
```

✅ Full functionality (may require enabling AUR in settings)

---

### Fedora

```bash
# Install Rust and build from source
sudo dnf install rust git cargo
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

⚠️ Limited functionality (Soar + Flatpak only, no AUR)

---

### Ubuntu/Debian

```bash
# Install Rust and build from source
sudo apt install rustc cargo git
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

⚠️ Limited functionality (Soar + Flatpak only, no AUR)

---

## Upgrading

### From AUR

```bash
paru -S declarch  # Will upgrade if already installed
# or
yay -S declarch
```

### From Source

```bash
cd declarch  # Your clone directory
git pull
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

---

## Uninstallation

### Remove from AUR

```bash
paru -Rns declarch
# or
yay -Rns declarch
```

### Remove Manual Installation

```bash
# Remove binary
sudo rm /usr/local/bin/declarch

# Remove config (optional)
rm -rf ~/.config/declarch
```

---

## Troubleshooting

### Issue: Command not found

**Symptom:** `bash: declarch: command not found`

**Solutions:**
1. Verify installation: `which declarch`
2. Check PATH: `echo $PATH`
3. If installed from source: `sudo install target/release/declarch /usr/local/bin/`
4. If installed via cargo: Add `~/.cargo/bin` to PATH

---

### Issue: Permission denied

**Symptom:** `bash: /usr/bin/declarch: Permission denied`

**Solution:**
```bash
sudo chmod +x /usr/bin/declarch
```

---

### Issue: Rust not found

**Symptom:** `bash: cargo: command not found`

**Solution:**
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or install via pacman
sudo pacman -S rust
```

---

### Issue: AUR helper not found

**Symptom:** AUR packages fail to install

**Solution:** Install an AUR helper first:

```bash
# Install paru
sudo pacman -S base-devel
git clone https://aur.archlinux.org/paru.git
cd paru
makepkg -si

# Or install yay
sudo pacman -S base-devel
git clone https://aur.archlinux.org/yay.git
cd yay
makepkg -si
```

---

## Next Steps

After installation:

1. **Read [Quick Start Guide](Quick-Start.md)** - Learn basic commands
2. **Configure declarch** - Edit `~/.config/declarch/declarch.kdl`
3. **Install packages** - Run `declarch sync`

---

## Need Help?

- **Documentation**: [Home](Home.md)
- **Issues**: https://github.com/nixval/declarch/issues
- **Discussions**: https://github.com/nixval/declarch/discussions
