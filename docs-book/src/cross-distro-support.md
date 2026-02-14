# Cross-Distro Support

Declarch is distro-agnostic, but support level depends on backend definitions available on your machine.

## Practical model

- Declarch itself is just the wrapper/orchestrator.
- Real package operations are done by backend binaries (`apt`, `dnf`, `pacman`, `flatpak`, `npm`, etc).
- You can mix multiple backends in one config.

## Common backend groups

- Arch-oriented: `aur`, `pacman`
- Debian/Ubuntu: `apt`, `nala`
- Fedora/RHEL: `dnf`
- Universal Linux: `flatpak`, `snap`, `nix`, `soar`
- Language/dev: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`

## Recommended starter setups

### Arch-based

```bash
declarch init
declarch init --backend paru,yay
```

### Debian/Ubuntu

```bash
declarch init
declarch init --backend apt,nala,npm,cargo
```

### Fedora

```bash
declarch init
declarch init --backend dnf,flatpak,npm,cargo
```

## Fallback examples

Backend definitions can include fallback behavior, for example:

- `nala -> apt`
- `pnpm -> npm`
- `yarn -> npm`
- `bun -> npm`
- `aur -> pacman`

This lets your config stay usable even when one preferred binary is missing.

## Contributing backend improvements

If your distro/backend combo needs better defaults, contribute to:

- https://github.com/nixval/declarch-packages
