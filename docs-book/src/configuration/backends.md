# Backends

Backends connect declarch to package managers.

## Built-in Backends

These work immediately after `declarch init`:

| Backend | Package Manager | Description |
|---------|-----------------|-------------|
| `aur` | paru/yay | AUR packages (falls back to pacman) |
| `pacman` | pacman | Official Arch repositories |
| `flatpak` | flatpak | Universal Linux apps |

## Using Backends

In your config:

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
✓ Backend 'npm' adopted!
```

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

| Backend | For |
|---------|-----|
| `npm` | Node.js packages |
| `cargo` | Rust crates |
| `pip` | Python packages |
| `soar` | Static binaries |

## Backend Storage

```
~/.config/declarch/
├── backends.kdl          # Built-in backends + imports
└── backends/
    └── npm.kdl           # Custom backend definitions
```

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

## Advanced

- [Create Custom Backends](../advanced/custom-backends.md)
