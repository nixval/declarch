# Changelog

All notable changes to this project are documented here.

This changelog is intentionally compact and focuses on what users need to migrate and operate safely.

## [Unreleased]

### Changed
- Documentation was aligned with the current v0.8 CLI flow and backend workflow.
- Command docs now consistently use `declarch info list ...` instead of old `declarch list ...` wording.
- Beginner docs were simplified to reduce stale syntax and reduce confusion between legacy and recommended patterns.

### Docs cleanup
- Removed obsolete command docs for removed paths.
- Consolidated list-related command guidance under `info` documentation.
- Updated README and docs-book navigation to match current behavior.

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
