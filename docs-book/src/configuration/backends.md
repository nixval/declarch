# Backends

Backends are how declarch talks to package managers.

Think of declarch as an **agnostic wrapper**:
- you write one declarative config,
- backends execute real package-manager commands.

## Built-in feeling vs reality

Declarch gives ready-to-use defaults, but backend files are still editable and can evolve.
This is intentional, so backend behavior can adapt when package managers change.

## Common backend set

You can use one or many:
- System: `aur`, `pacman`, `flatpak`, `apt`, `nala`, `dnf`, `snap`, `nix`, `brew`
- Language/dev: `npm`, `pnpm`, `yarn`, `bun`, `cargo`, `pip`, `gem`, `go`
- Others: `soar`

## Add backend definitions

```bash
declarch init --backend npm
```

Multiple at once:

```bash
declarch init --backend pnpm,yarn
# or
declarch init --backend pnpm yarn
```

## Example usage in config

```kdl
pkg {
    pacman { firefox }
    flatpak { org.mozilla.firefox }
    npm { typescript prettier }
    nix { nil }
}
```

## Fallback idea (important)

Some backends can fallback to another backend when binary is missing.
Examples:
- `nala -> apt`
- `yarn -> npm`
- `pnpm -> npm`
- `bun -> npm`
- `aur/yay/paru -> pacman` (depending on backend config)

## Practical advice

- Start with minimal backends first.
- Add more only when you really use them.
- Keep backend files versioned so you can track behavior changes.
