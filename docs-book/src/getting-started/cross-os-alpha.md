# Cross-OS (Alpha)

Declarch is designed to stay backend-agnostic, but cross-OS support is still in progress.

Current status:
- Linux: most tested path
- macOS: alpha preview
- Windows: alpha preview

This is still usable for early testing.
Start with preview/doctor first.

## Quick safety flow

```bash
declarch info --doctor
declarch sync preview
```

`info --doctor` prints the actual config/state paths for your current OS.

## Paths by OS

Declarch uses platform-native directories via Rust `ProjectDirs`.

Typical examples:
- Linux config: `~/.config/declarch`
- Linux state: `~/.local/state/declarch/state.json`
- macOS config: `~/Library/Application Support/declarch`
- macOS state: `~/Library/Application Support/declarch/state/state.json` (platform/runtime-dependent)
- Windows config/state: under `%APPDATA%` / `%LOCALAPPDATA%` equivalents (platform/runtime-dependent)

Use `declarch info --doctor` for exact paths on your machine.

## Why this still works for beginners

- If a backend is not available for your OS, declarch warns and skips.
- Your config stays declarative and portable.
- You can still run the same `declarch sync` flow on different machines.

## Roadmap direction

- Better Windows backend coverage (`winget`, `choco`, `scoop`) is planned.
- More macOS validation (`brew` + mixed backends) is planned.
- Warnings are designed to stay clear and beginner-friendly.
