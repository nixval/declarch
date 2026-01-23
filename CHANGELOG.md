# Changelog

All notable changes to this project will be documented in this file.

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

## [0.4.1] - 2025-01-20

### Added
- Generic backend system.
- User-defined backends via `backends.kdl`.
- Conflict detection.
