# Changelog

All notable changes to this project are documented here.

This changelog is intentionally compact and focuses on what users need to migrate and operate safely.

## [Unreleased]

### Added
- Lightweight performance baseline harness script (`scripts/perf_baseline.sh`) for repeatable hot-path checks.
- Beginner onboarding docs:
  - `First Run (Linear Guide)`
  - `Config Progression (Minimal -> Advanced)`
  - `Common Mistakes`
- Release governance docs:
  - `RELEASE_CHECKLIST.md`
  - `plan/rollback-map.md`

### Changed
- Sync dry-run lock behavior no longer holds the state lock for the full command duration.
- Sync executor now avoids Rayon overhead for very small backend sets by using a sequential path when applicable.
- CLI help output now includes a clearer quick-start flow for first-time users.
- Error messages for common mistakes (for example invalid `init --list` target or unsupported machine-output command) now include direct next-step guidance.
- CI quality gates are now aligned with project phase criteria:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --all-targets`
- Release script checks now enforce changelog discipline for `[Unreleased]`.
- `sync prune` (non-dry-run) now uses strict state loading and fails fast if state recovery fails, instead of silently continuing with default state.

### Security
- Remote init/fetch defaults were hardened to prefer HTTPS and require explicit opt-in for insecure HTTP (`DECLARCH_ALLOW_INSECURE_HTTP=1`).

### Fixed
- Corrected shell single-quote escaping behavior in sanitization path.
- Search `--limit` now fails fast on invalid values instead of silently falling back.
- Private network range validation for `172.16.0.0/12` was corrected.
- Remote fetch failure diagnostics now include attempted URL summary for faster troubleshooting.

## [0.8.0] - 2026-02-10

### ⚠ Breaking changes

- Major architecture refactor for backend loading and execution.
- CLI command shape changed for sync variants.
- Old docs/snippets from pre-0.8 may not work without migration.

### Added
- Unified backend management via generic backend execution path.
- Stronger backend metadata and init/adoption flow.
- Improved error reporting with clearer parse and config context.
- Multi-backend initialization workflow.

### Changed
- Sync command variants use subcommands:
  - `declarch sync preview`
  - `declarch sync update`
  - `declarch sync prune`
- Config and backend workflows were redesigned to be explicit and more maintainable.
- Backend definitions became easier to evolve as package-manager behavior changes.

### Fixed
- Multiple backend parser and search-format issues in official/remote backend definitions.
- Better rollback behavior when installation flow is interrupted.
- Better validation for missing backends and malformed backend configs.
- Safer handling around command execution and configuration updates.

### Migration quick steps

```bash
# 1) Backup
cp -r ~/.config/declarch ~/.config/declarch.backup

# 2) Re-initialize base files
# (or migrate manually if you maintain custom setup)
declarch init

# 3) Review config and backend definitions, then preview
declarch sync preview

# 4) Apply
declarch sync
```

## Older history

Older per-commit details before `0.8.0` are available in git history and tags.
This file intentionally keeps only the high-signal migration/operator view.
