# Cross-Distro Support

Declarch works on any Linux distribution with the right backends.

## Built-in Backends

| Distro | aur | pacman | flatpak |
|--------|-----|--------|---------|
| Arch | ✓ | ✓ | ✓ |
| EndeavourOS | ✓ | ✓ | ✓ |
| Manjaro | ✓ | ✓ | ✓ |
| Debian/Ubuntu | ✗ | ✗ | ✓ |
| Fedora | ✗ | ✗ | ✓ |

## Universal Backends

These work on all distros:

- `flatpak` - Universal apps
- `npm` - Node.js packages
- `cargo` - Rust crates
- `pip` - Python packages
- `soar` - Static binaries

## Adding Distro Support

Need apt, dnf, or zypper? Create a custom backend:

```kdl
// backends/apt.kdl
backend "apt" {
    binary "apt"
    
    list "apt list --installed" {
        format "regex"
        regex {
            pattern "^([^/]+)"
            name_group 1
        }
    }
    
    install "apt install -y {packages}"
    remove "apt remove -y {packages}"
    needs_sudo "true"
}
```

See [Custom Backends](advanced/custom-backends.md) for details.

## Recommended Setup by Distro

### Arch-based

```bash
declarch init
# Use aur, pacman, flatpak
```

### Debian/Ubuntu

```bash
declarch init --backend npm
declarch init --backend cargo
# Use flatpak, npm, cargo, custom apt backend
```

### Fedora

```bash
declarch init --backend npm
declarch init --backend cargo
# Use flatpak, npm, cargo, custom dnf backend
```

## Contributing Backends

Submit backends for your distro to the [declarch-packages](https://github.com/nixval/declarch-packages) repository.
