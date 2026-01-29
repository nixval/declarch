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

### Added
- **Install command**: Declarative package installation with automatic sync
  - `declarch install <package>` - Add package to modules/others.kdl and auto-sync
  - `declarch install <pkg1> <pkg2> <pkg3>` - Install multiple packages at once
  - `declarch install soar:bat` - Backend-specific package installation
  - `declarch install <pkg> --modules <name>` - Install to specific module
  - `--no-sync` flag to skip automatic sync
  - **Cross-backend duplicate detection**: Prompts user when package exists in different backend
  - Example: `bat (AUR)` exists → User runs `declarch install soar:bat` → Prompts "Install from Soar anyway? [y/N]"
  - Smart skipping: Warns and skips if exact package (same name + backend) already exists
- **Settings command**: Global configuration management
  - `declarch settings set <key> <value>` - Set a configuration value
  - `declarch settings get <key>` - Get a configuration value
  - `declarch settings show` - Show all settings
  - `declarch settings reset <key>` - Reset setting to default
  - **Color settings**: `color` (auto/always/never) - Now properly integrated!
  - **Editor setting**: `editor` - Set default editor via command line
  - Other settings: `progress` (on/off), `format` (table/json/yaml), `verbose` (true/false)
- **Config editor module** (`src/config/editor.rs`): Programmatic KDL file editing
  - Add packages to KDL configuration files
  - Create new module files with proper structure
  - Support for nested module paths (e.g., `linux/notes`)
  - Package string parsing (`backend:package` format)

### Fixed
- Removed obsolete `dcl` alias references from README.md
- **Color settings now work**: Previously were stored but never applied to output
  - Added `atty` crate dependency for TTY detection
  - Integrated color mode checking into all UI functions (success, error, warning, info, header, etc.)
  - Settings loaded once at startup via `init_colors()`
  - Modes: `auto` (TTY-aware), `always` (force colors), `never` (plain text)
- **KDL syntax fix**: Package insertion now produces valid KDL
  - Fixed package insertion logic to properly place packages inside braces
  - Before: `packages {\n} bat` (invalid - package outside braces)
  - After: `packages {\n  bat\n}` (valid - package inside braces)
- **Auto-import for new modules**: Install command now auto-imports newly created modules
  - When creating `modules/others.kdl`, automatically adds import to `declarch.kdl`
  - Ensures sync can find packages in newly created modules
  - Uses same regex-based injection logic as `init` command
- Install command now properly detects existing packages in config to prevent duplicates
- Backend string conversion fix for package duplicate checking

### Changed
- **Editor priority**: Settings > KDL config > $EDITOR > $VISUAL > nano (default)
  - Can now set editor via: `declarch settings set editor nvim`
  - Previously only configurable via KDL config or environment variables
- Refactored `sync.rs` to move hook logic to `commands/hooks.rs`.
- Deduplicated pre/post sync hook execution logic.

### Security
- Added input sanitization for package names to prevent shell injection.
- Enhanced SSRF protection in remote module (checking Link-local and IPv6 ranges).
- Refactored hook execution to parse arguments safely instead of resolving strings in shell.

## [0.4.2] - 2025-01-20

### Added
- Generic backend system.
- User-defined backends via `backends.kdl`.
- Conflict detection.
