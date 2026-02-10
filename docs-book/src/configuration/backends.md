# Backends

Backends connect declarch to package managers. They define how to list, install, remove, and search packages for each package manager.

## Built-in Backends

These work immediately after `declarch init`:

| Backend | Package Manager | Description |
|---------|-----------------|-------------|
| `aur` | paru/yay | AUR packages (falls back to pacman) |
| `pacman` | pacman | Official Arch repositories |
| `flatpak` | flatpak | Universal Linux apps |

## Using Backends

In your config (`declarch.kdl`):

```kdl
pkg {
    aur {
        neovim
        brave-bin
    }
    
    pacman {
        firefox
        thunderbird
    }
    
    flatpak {
        com.spotify.Client
        com.discordapp.Discord
    }
}
```

## Installing Custom Backends

Need npm, cargo, or others?

```bash
declarch init --backend npm
```

This will:
1. Fetch the backend definition from the registry
2. Show meta information (title, description, platforms, etc.)
3. Prompt for confirmation
4. Import the backend automatically

Then use it:
```kdl
pkg {
    npm {
        typescript
        prettier
    }
}
```

## Available Backends

Install these with `declarch init --backend <name>`:

| Backend | For | Platforms |
|---------|-----|-----------|
| `npm` | Node.js packages | Linux, macOS, Windows |
| `pnpm` | Fast Node.js package manager | Linux, macOS, Windows |
| `bun` | Fast JavaScript runtime | Linux, macOS |
| `yarn` | Alternative Node.js package manager | Linux, macOS, Windows |
| `cargo` | Rust crates | Linux, macOS, Windows |
| `pip` | Python packages | Linux, macOS, Windows |
| `gem` | Ruby gems | Linux, macOS, Windows |
| `soar` | Static binaries | Linux |
| `nix` | Nix packages | Linux, macOS |
| `brew` | Homebrew packages | macOS, Linux |
| `apt` | Debian/Ubuntu packages | Linux |
| `dnf` | Fedora/RHEL packages | Linux |
| `snap` | Universal Linux packages | Linux |
| `paru` | AUR helper (alternative) | Linux |
| `yay` | AUR helper (alternative) | Linux |
| `go` | Go binaries | Linux, macOS, Windows |

## Backend Storage

```
~/.config/declarch/
├── backends.kdl          # Built-in backends + imports
└── backends/
    └── npm.kdl           # Custom backend definitions
```

## Backend Meta Information

When initializing a backend, declarch displays meta information:

```
fetching 'npm' from nixval/declarch-packages

  Title:       NPM
  Description: Node Package Manager
  Maintained:  nixval
  Homepage:    https://www.npmjs.com
  Platforms:   linux, macos, windows
  Requires:    nodejs

? Are you sure you want this 'npm' being adopted [Y/n]
```

Fields with value "-" are hidden automatically.

## Troubleshooting

**"Backend 'xxx' not found"**

```bash
# Install the backend
declarch init --backend xxx
```

**"No backend configured"**

```bash
# Re-initialize
declarch init
```

**"Backend file already exists"**

```bash
# Force overwrite
declarch init --backend xxx --force
```

## Advanced

- [Create Custom Backends](../advanced/custom-backends.md)
