# Changelog

All notable changes to this project will be documented in this file.

## [0.5.1] - 2025-01-29

### Security
- **CRITICAL**: Tightened hook command validation regex to prevent command injection attacks.
  - Old: `^[\w\s\-\./@:=\$~\{\}]+$` (too permissive)
  - New: `^[a-zA-Z0-9_\-\.\s/:]+$` (more restrictive)
  - Removed dangerous characters: `$`, `{`, `}`, `~`, `=`, `@`
- Documented safety rationale for `unsafe` block in info.rs (required by Rust 1.92+)
- Added documentation about state rollback limitations in switch.rs

### Performance
- Reduced memory allocations in `sync.rs` by using iterators instead of cloning vectors
  - Optimized `update_state_after_sync` to chain iterators instead of cloning Vec<PackageId>
  - Eliminated 3 vector clones per sync operation

### Fixed
- Improved error messages in hook validation to show allowed characters
- Added comprehensive safety comments for atomic state operations

### Code Quality
- Verified `kdl.rs` structure: 1,422 lines total (979 lines tests, 443 lines production code)
- Confirmed `sync.rs` is well-organized with helper functions (847 lines total)

## [Unreleased]

### Security
- Added input sanitization for package names to prevent shell injection.
- Enhanced SSRF protection in remote module (checking Link-local and IPv6 ranges).
- Refactored hook execution to parse arguments safely instead of resolving strings in shell.

### Added
- `--no-auto-import` prompt in `init` command to ask for consent before modifying `declarch.kdl`.
- `shell-words` dependency for safe command parsing.
- CI/CD workflow for automated testing and auditing.

### Changed
- Refactored `sync.rs` to move hook logic to `commands/hooks.rs`.
- Deduplicated pre/post sync hook execution logic.

## [0.4.2] - 2025-01-20

### Added
- Generic backend system.
- User-defined backends via `backends.kdl`.
- Conflict detection.
