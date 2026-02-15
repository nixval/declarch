# explain

Explain why a package/target is part of your current declarch setup.

## Usage

```bash
declarch explain <query>
declarch explain --target sync-plan
```

## Examples

```bash
declarch explain bat
declarch explain aur:bat
declarch explain --target sync-plan
declarch explain bat --profile desktop --host vps-1
```

## What it shows

- active context (`profile`, `host`, extra modules)
- package backend identity
- source files where package is declared
- installed state hints from state database
- cross-backend name collisions

## Notes

- `--target sync-plan` gives high-level install/remove drift summary.
- profile/host overlays are opt-in and only applied when passed explicitly.
