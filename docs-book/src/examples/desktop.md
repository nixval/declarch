# Desktop Environment Setup

Complete desktop configuration with multiple package managers.

## Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
  host "desktop-arch"
}

// System packages (pacman/AUR)
packages {
  // Window manager
  hyprland
  waybar
  rofi-wayland
  waypaper

  // Terminal
  alacritty
  zsh
  starship

  // Applications
  firefox
  thunar
  discord
  slack
}

// Development tools
packages:cargo {
  ripgrep
  fd-find
  zoxide
  bat
  exa
}

packages:npm {
  typescript
  prettier
  eslint
}

// Flatpak applications
flatpak {
  com.spotify.Client
  org.telegram.desktop
  com.discordapp.Discord
}

// Optional hooks
hooks {
  post-sync "echo 'Desktop environment synced!' && notify-send 'Declarch' 'Packages synced successfully'"
}
```

## Features

- **Hyprland** as Wayland compositor
- **Alacritty** terminal with **Zsh** + **Starship** prompt
- Development tools via Cargo and npm
- GUI applications via Flatpak
- Desktop notifications after sync

## What This Includes

### Core Desktop
- Window manager: Hyprland
- Status bar: Waybar
- Launcher: Rofi
- Wallpaper: Waypaper

### Terminal
- Terminal emulator: Alacritty
- Shell: Zsh
- Prompt: Starship

### Applications
- Browser: Firefox
- File manager: Thunar
- Communication: Discord, Slack, Telegram

### Development
- Rust tools: ripgrep, fd, zoxide, bat, eza
- Node.js tools: TypeScript, Prettier, ESLint

### Flatpak
- Spotify
- Telegram Desktop

## Usage

```bash
# Check configuration
declarch check

# Dry run to see what will change
declarch sync --dry-run

# Sync all packages
declarch sync
```

## Source Files

- [`desktop.kdl`](https://github.com/nixval/declarch/blob/main/examples/desktop.kdl)

---

**Next:** See [Development Environment](development.html) for programming-focused setup.
