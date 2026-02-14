# Backends

Backends are how declarch talks to package managers.

Think of declarch as an agnostic wrapper:
- you declare packages once,
- backend definitions run real package-manager commands.

## Default + adopt model

- `declarch init` creates default backend definitions.
- `declarch init --backend <name>` adopts extra backend files from remote registry.
- backend files stay editable locally so behavior can evolve with upstream tools.

## Common backend set

- System: `aur`, `pacman`, `flatpak`, `apt`, `nala`, `dnf`, `snap`, `nix`, `brew`
- Language/dev: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`
- Others: `soar`

## Add backend definitions

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid
declarch init --backend pnpm yarn
```

## Example package config

```kdl
pkg {
    pacman { firefox }
    flatpak { org.mozilla.firefox }
    npm { typescript prettier }
    nix { nil }
}
```

## Fallback idea

Backends can fallback when a binary is missing.

Examples:
- `nala -> apt`
- `yarn -> npm`
- `pnpm -> npm`
- `bun -> npm`
- `aur -> pacman`

## Practical advice

- Start with minimal backends.
- Add more only when needed.
- Keep backend files in version control.
