# Cross-Distro Support

Declarch is distro-agnostic.
Real behavior depends on which backend binaries and backend configs are available.

## How to think about it

- Declarch = coordinator.
- Backends = actual package-manager commands.
- You can mix distro/system + language backends in one file.

## Common backend groups

- Arch-oriented: `aur`, `pacman`
- Debian/Ubuntu: `apt`, `nala`
- Fedora/RHEL: `dnf`
- Universal: `flatpak`, `snap`, `nix`, `soar`
- Language/dev: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`

## Starter commands

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

- `nala -> apt`
- `pnpm -> npm`
- `yarn -> npm`
- `bun -> npm`
- `aur -> pacman`

Fallback keeps workflows usable when preferred binary is missing.

## Improve support

Contribute backend definitions and fixes:
- https://github.com/nixval/declarch-packages
