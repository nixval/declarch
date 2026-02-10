# Changelog

All notable changes to this project will be documented in this file.

## [0.8.0] - 2026-02-10

### ⚠️ BREAKING CHANGES

This is a major refactor with breaking changes. Expect errors when upgrading from v0.5.x or earlier.

#### New Architecture
- **Backend system rewritten**: Official backends (aur, pacman, flatpak) now built into backends.kdl
- **Import pattern**: Custom backends use `imports { "backends/name.kdl" }` pattern
- **No auto-loading**: Backends are no longer auto-loaded from directories
- **Generic backend manager**: Unified backend implementation via `GenericManager`

#### New KDL Syntax
- Use `pkg` instead of `packages`
- All string values must be quoted: `format "whitespace"`, `needs_sudo "true"`
- Backend blocks go inside `pkg { }`
- Editor configuration in declarch.kdl: `editor "vim"`

#### New CLI Structure
- `declarch sync --update` → `declarch sync update`
- `declarch sync --prune` → `declarch sync prune`
- `declarch sync --dry-run` → `declarch sync preview`

### Added
- **Enhanced error reporting**: Rust-like error messages with line numbers and hints
- **Backend import system**: Explicit imports for custom backends
- **Official backends embedded**: aur, pacman, flatpak work out of the box
- **Meta info display**: Shows backend metadata (title, description, platforms, etc.) during init
- **Multi-backend init**: Initialize multiple backends at once: `declarch init --backend npm --backend cargo`
- **Force flag for backend init**: `--force` to overwrite existing backend files
- **Editor setting in KDL**: Configure editor in declarch.kdl with `editor "vim"`

### Changed
- **Simplified CLI**: Removed duplicate flags, standardized naming
- **Better error messages**: File path, line:column, visual indicators
- **Backend init multiple flags**: Use multiple `--backend` flags instead of comma-separated
- **Template updated**: New declarch.kdl template with v0.8.0 syntax

### Fixed
- **Backend format compatibility**: Fixed soar, pnpm, bun backend configurations
  - soar: Uses `json_lines` format for NDJSON output
  - pnpm: Uses `regex` for list, `npm_json` for search
  - bun: Uses `regex` for tree output parsing
- **JSON parser robustness**: Skip invalid JSON lines in NDJSON output
- **Backend validation**: Proper error messages for missing `name_key` in JSON formats

### Fixed
- **Search output formatting**: Fixed AUR and Flatpak search configurations
  - AUR (`paru -Ss`): Uses `format "whitespace"` with proper column mapping
    - `name_col 0`: repo/package name
    - `desc_col 2`: description (skipping version column)
  - Flatpak: Uses TSV format with Application ID as name for easy config reference
- **Config rollback on cancel**: Fixed rollback when user cancels sync during install
  - Install command now properly rolls back config changes when user cancels
  - Returns Interrupted error on sync cancel to trigger rollback
  - Shows "Changes rolled back" message to confirm
- **Backend validation**: Validate backend exists before adding package to config
  - Prevents config pollution from typos like 'uar' instead of 'aur'
  - Shows warning: "Backend 'xxx' not found. Run 'declarch init --backend xxx'"
- **NPM error output**: Use --silent flag to reduce verbose npm error messages
  - NPM backend now uses --silent for install/remove commands
  - Cleaner error messages when packages don't exist

### Migration Guide
```bash
# Backup your config
cp -r ~/.config/declarch ~/.config/declarch.backup

# Re-initialize
declarch init

# Fix your .kdl files:
# - Change 'packages {' to 'pkg {'
# - Change 'format whitespace' to 'format "whitespace"'
# - Change 'needs_sudo true' to 'needs_sudo "true"'

# Sync
declarch sync
```

## [0.5.2] - 2026-01-31

### Added
- **Multi-backend search**: Comprehensive search support across all package managers
  - AUR, Flatpak, Soar, npm, yarn, pnpm, bun, cargo, brew - all with search!
  - Real package count with limit notation: "Found 42 packages matching 'rust' --limit 10 (showing 10):"
  - Default limit of 10 results per backend (configurable with --limit)
  - Unlimited results with `--limit all` or `--limit 0`
- **Custom backend search**: Configure search in backends.kdl with 4 format types
  - JSON format: Parse API responses with json_path, name_key, desc_key
  - Tab-separated format: For CLI tools with tab output
  - Whitespace-separated format: For space-separated output
  - Regex format: Extract info with custom regex patterns
- **Non-Arch distro detection**: Warn users when searching AUR from non-Arch systems
  - Auto-detects distro from /etc/os-release
  - Shows warning: "You are using a non-Arch based distro (Debian). Searching AUR may not work..."
  - Provides helpful tips to use other backends
- **Backend-specific search syntax**: `declarch search npm:prettier` to search specific backend
- **Search result limiting**: Centralized limiting for consistent behavior across backends

### Changed
- **Auto mode default**: Now searches AUR only by default (not all backends)
  - Use `--backends` flag or `backend:query` syntax for other backends
  - More predictable and focused default behavior
- **Error messages**: Better warnings for backends without search support
  - Custom backends: "Search from custom backend 'my-pm' is not working. Add 'search' configuration..."
  - Built-in backends: Clear list of supported backends

### Fixed
- **Result count display**: Now shows actual total count before limiting
  - Previously: Always showed "Found 10 packages" (wrong)
  - Now: "Found 42 packages matching 'rust' --limit 10 (showing 10):" (correct)

## [0.5.1] - 2026-01-30

### Security
- **CRITICAL**: Tightened hook command validation regex to prevent command injection attacks
  - Old: `^[\w\s\-\./@:=\$~\{\}]+$` (too permissive)
  - New: `^[a-zA-Z0-9_\-\.\s/:]+$` (more restrictive)
  - Removed dangerous characters: `$`, `{`, `}`, `~`, `=`, `@`
- **CRITICAL**: Added state JSON validation before write
  - Validates JSON structure before writing to prevent corruption
  - Prevents truncated state files from crashes
- State file corruption protection with JSON validation
- Improved backup integrity checks
- Documented safety rationale for `unsafe` block in info.rs (required by Rust 1.92+)
- Added documentation about state rollback limitations in switch.rs

### Performance
- Reduced memory allocations in `sync.rs` by using iterators instead of cloning vectors
  - Optimized `update_state_after_sync` to chain iterators instead of cloning Vec<PackageId>
  - Eliminated 3 vector clones per sync operation

### Added
- **Horizontal package display**: Packages grouped by backend with auto-wrap
  - `aur: bat hyprland waybar`
  - `flatpak: com.spotify.Client org.mozilla.firefox`
  - Auto-wraps based on terminal width for better readability
- **Custom backend documentation**: Comprehensive guide for distro-specific backends
  - Explains how to use nala, dnf5, zypper, and other distro-specific package managers
  - Warnings that custom backends are unofficial and fragile
  - Complete examples and setup instructions
- **Cross-distro support documentation**: Matrix of backend availability across distributions
  - Documents which backends work on Arch, Debian, Fedora, and other distros
  - Explains AUR limitations on non-Arch systems
- **Compact mode setting**: User preference for compact output
  - `declarch settings set compact true` for concise output
- **Selective module sync**: `--module` flag now syncs only specified module
  - `declarch install bat --module base` - Installs and syncs only base module
  - More efficient: No longer syncs all modules when installing to specific one
- **Automatic rollback on failure**: Failed installations restore KDL files from backup
  - Creates timestamped backup before modifying KDL files
  - Automatically restores on sync failure
  - Cleans up backups on successful install
- **Package backend display**: Transaction plans now show `(backend)` info
  - Example: `Changes: Install: bat (aur), vim (soar)`
  - Clear visibility into which backend packages come from
- **Compact UI output**: Simplified init and install messages
  - Removed verbose "Trying:" messages during module fetch
  - Removed "Synchronizing Packages" and "Scanning system state" headers
  - Removed separator lines for cleaner output
  - Clearer success messages: "Sync completed, added to 'module.kdl'"
- **Comprehensive package string validation**: Prevents malformed inputs
  - Validates: Empty strings, multiple colons, empty backend/package
  - Clear error messages for invalid formats

### Changed
- **UI is more concise**: Less overwhelming output
  - Removed "Entry point" messages in check command
  - Removed ✓/ℹ symbols from success/info messages (kept ⚠/✗ for attention)
  - Reduced separator usage throughout
  - Init: "fetch: URL" instead of multiple "Trying:" messages
  - Install: Shows package list with backend in one line
  - Sync: Direct to changes, no intermediate headers
  - Error messages simplified: Only essential information shown
- **README title**: Changed from "Declarative Package Manager for Arch Linux" to "for Linux"
- **Cross-distro clarity**: Custom backends now documented in README and dedicated guide
- "missing import" warnings now respect verbose setting
- Removed verbose rollback messages
- Removed "Please check error messages above" suffix

### Removed
- **Deprecated KDL editor syntax**: `editor "nvim"` in declarch.kdl files
  - Use `declarch settings set editor nvim` instead
  - Or set `$EDITOR` / `$VISUAL` environment variables
  - Settings system was already taking precedence, this removes the old syntax
  - Removed KDL parsing code for editor node
  - Removed editor field from RawConfig struct
  - Updated editor resolution to only use settings and environment variables
- **Obsolete documentation files**:
  - RELEASE.md (use CHANGELOG.md instead)
  - HOOKS-BRAINSTORM.md, HOOKS-SYNTAX-PROPOSAL.md, HOOKS-SYNTAX-FINAL.md (hooks system is implemented and documented in docs-book)
  - Simplified CONTRIBUTING.md to reference full documentation
  - Updated release workflow to use CHANGELOG.md

### Fixed
- **CRITICAL**: Fixed panic when package exists in multiple backends without --backend flag
  - install.rs:95 - Backend unwrap now defaults to "aur" instead of panicking
- **CRITICAL**: Fixed 4 panic risks in editor.rs path operations
  - Invalid module paths now return proper errors instead of panicking
  - Invalid backup paths now handled gracefully
- **Python backend naming**: Changed `packages:python` to `packages:pip` in examples
  - Backend is named `PipParser` in code, syntax should match
  - No functional change, just naming consistency
  - Updated all example files to use `packages:pip`
- **Fixed Windows path handling**: Cross-platform path normalization
  - Uses Path components instead of simple string replace
  - Handles mixed path separators correctly
- **Added cleanup error logging**: Failed cleanup operations now show warnings
  - No more silent failures when removing backup files
- All compiler warnings resolved (unused variables)
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
