# Introduction

**Declarch** is a declarative package manager for Linux.

Instead of running `pacman -S`, `npm install`, and `flatpak install` commands every time, you declare all your packages in one config file and run a single command to sync.

## What Does That Mean?

**Traditional way:**
```bash
# You install things one by one
paru -S neovim bat fzf
npm install -g typescript prettier
flatpak install com.spotify.Client

# Months later, you forget what you installed and why
```

**Declarch way:**
```kdl
// ~/.config/declarch/modules/base.kdl
pkg {
    aur {
        neovim
        bat
        fzf
    }
    
    npm {
        typescript
        prettier
    }
    
    flatpak {
        com.spotify.Client
    }
}
```

Then run:
```bash
declarch sync
```

Your system now matches your config. Add this file to Git and use it on all your machines.

## 30-Second Demo

```bash
# 1. Install
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh

# 2. Initialize
declarch init

# 3. Add some packages
declarch install bat fzf ripgrep

# 4. Sync
declarch sync
```

Done! Your packages are now managed declaratively.

## Key Features

| Feature | What It Means |
|---------|---------------|
| **Declarative** | Define what you want, not how to get it |
| **Multi-Backend** | One config for AUR, Flatpak, npm, cargo, etc. |
| **Modular** | Split into multiple files (base, gaming, work) |
| **Git-Friendly** | Track your setup in version control |

## Who Is This For?

- **Arch Linux users** who want reproducible setups
- **Developers** using multiple package managers
- **Multi-machine users** who want consistent environments
- **Anyone** tired of forgetting what they installed

## Requirements

- Linux (Arch-based distros work best)
- One of: `paru`, `yay`, or `pacman` installed

## Next Steps

→ [Installation](./getting-started/installation.md)  
→ [Quick Start Guide](./getting-started/quick-start.md)
