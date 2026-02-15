# Backends

Backends are how declarch talks to package managers.

Think of it like this:
- declarch is the coordinator,
- backends do the real package operations.

## Default + adopt model

- `declarch init` creates default backend definitions.
- `declarch init --backend <name>` adopts extra backends from registry.
- Local backend files are editable.

## Common backend groups

- System: `aur`, `pacman`, `flatpak`, `apt`, `nala`, `dnf`, `snap`, `nix`, `brew`
- Language/dev: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`
- Other: `soar`

## Add backend definitions

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
declarch init --backend pnpm yarn
```

## Use backends in package config

```kdl
pkg {
    pacman { firefox }
    flatpak { org.mozilla.firefox }
    npm { typescript }
    nix { nil }
}
```

## Fallback concept

A backend can fallback when binary is missing.
Examples:
- `nala -> apt`
- `pnpm -> npm`
- `yarn -> npm`
- `bun -> npm`
- `aur -> pacman`

## Beginner tips

- Start small.
- Add one backend at a time.
- Keep backend files versioned in git.
