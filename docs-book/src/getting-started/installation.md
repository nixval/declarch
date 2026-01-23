# Installation Guide

Complete guide to installing declarch on Linux.

## Requirements

### Arch Linux / Arch-based Distributions

**Required:**
- An AUR helper: `paru` or `yay`
- `git` (for remote init)

### Other Distributions

**Required:**
- One or more package managers that declarch will manage
- Supported backends for cross-distro: Flatpak, npm, pip, cargo, brew

## Installation Methods

### From AUR (Recommended for Arch)

**Build from source:**
```bash
paru -S declarch
```

**Pre-built binary:**
```bash
paru -S declarch-bin
```

### Install Script

Downloads the latest binary and installs to `/usr/local/bin/`:

```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### Build from Source

```bash
# Clone repository
git clone https://github.com/nixval/declarch.git
cd declarch

# Build release binary
cargo build --release

# Install to system
sudo install target/release/declarch /usr/local/bin/

# Verify installation
declarch --version
```

## Post-Installation

### 1. Verify Installation

```bash
declarch --version
# Output: declarch 0.4.2
```

### 2. Initialize Configuration

```bash
declarch init
```

This creates:
- `~/.config/declarch/declarch.kdl` - Main configuration
- `~/.local/state/declarch/state.json` - Package state tracking

### 3. Install Required Package Managers

Declarch does NOT install package managers for you. Install the ones you need:

**For Arch-based systems:**
```bash
# AUR helper (required for AUR backend)
paru -S paru  # or: paru -S yay

# Optional package managers
paru -S flatpak npm python pip cargo
```

**For other distributions:**
```bash
# Install your preferred package managers
# Example for Ubuntu/Debian:
sudo apt install flatpak npm python3-pip cargo
```

### 4. Enable Shell Completions

**Bash:**
```bash
# Add to ~/.bashrc
declarch completions bash > ~/.local/share/bash-completion/completions/declarch
source ~/.local/share/bash-completion/completions/declarch
```

**Zsh:**
```bash
# Add to ~/.zshrc
declarch completions zsh > ~/.local/share/zsh/site-functions/_declarch
```

**Fish:**
```bash
# Add to ~/.config/fish/completions/declarch.fish
declarch completions fish > ~/.config/fish/completions/declarch.fish
```

## Upgrading

### From AUR

```bash
paru -S declarch
```

### Using Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### From Source

```bash
cd declarch
git pull
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

## Uninstallation

### Remove Binary

```bash
sudo rm /usr/local/bin/declarch
sudo rm /usr/local/bin/dc  # if you created the alias
```

### Remove Configuration and State

```bash
# Remove config directory
rm -rf ~/.config/declarch

# Remove state directory
rm -rf ~/.local/state/declarch

# Remove completions
rm ~/.local/share/bash-completion/completions/declarch
rm ~/.local/share/zsh/site-functions/_declarch
rm ~/.config/fish/completions/declarch.fish
```

## Troubleshooting

### "Command not found: declarch"

**Cause:** Binary not in PATH

**Solution:**
1. Verify installation: `ls /usr/local/bin/declarch`
2. Check PATH: `echo $PATH`
3. If missing, add to shell config:
   ```bash
   export PATH="/usr/local/bin:$PATH"
   ```

### "Permission denied" when running declarch

**Cause:** Some operations require sudo access (e.g., AUR package installation)

**Solution:** Use sudo for commands that need it:
```bash
sudo declarch sync
```

### State file errors

**Cause:** Corrupted state file

**Solution:**
```bash
# View backup
cat ~/.local/state/declarch/state.json.backup

# Restore from backup
cp ~/.local/state/declarch/state.json.backup ~/.local/state/declarch/state.json

# Or start fresh (will re-detect installed packages)
rm ~/.local/state/declarch/state.json
declarch sync
```

## Next Steps

After installation, continue to [Quick Start Guide](quick-start.md) to set up your first configuration.
