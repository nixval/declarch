# Declarch Development Guidelines

## Project Overview

**Declarch** is a universal declarative package manager for Linux, supporting multiple backends (AUR, Flatpak, npm, cargo, pip, brew, and custom).

- **Version**: 0.5.2
- **Language**: Rust (Edition 2024)
- **Repository**: https://github.com/nixval/declarch

## Architecture

### Module Structure

```
src/
├── main.rs              # Entry point (31 lines)
├── lib.rs               # Library exports
├── cli/                 # CLI parsing & dispatch
│   ├── args.rs          # Clap argument definitions
│   ├── dispatcher.rs    # Command routing
│   └── deprecated.rs    # Legacy flag handling
├── commands/            # Command implementations
│   ├── sync/            # Sync workflow (6 modules)
│   │   ├── mod.rs       # Orchestration
│   │   ├── planner.rs   # Transaction planning
│   │   ├── executor.rs  # Package execution
│   │   ├── state_sync.rs # State updates
│   │   ├── hooks.rs     # Hook execution
│   │   ├── variants.rs  # Variant matching
│   │   └── diff.rs      # Diff display
│   ├── info.rs          # Info command
│   ├── info/            # Info submodules
│   │   └── summary.rs   # Summary display
│   ├── init.rs          # Init command
│   ├── check.rs         # Check command
│   └── ...              # Other commands
├── config/              # Configuration handling
│   ├── kdl.rs           # KDL facade (31 lines)
│   └── kdl_modules/     # KDL parsing modules
├── core/                # Core types & logic
├── packages/            # Package manager backends
├── state/               # State management
├── backends/            # Backend abstractions
├── utils/               # Utilities
├── ui/                  # UI components
└── constants/           # Constants
```

## Development Tools

### Required Tools

```bash
# Install development dependencies
cargo install cargo-audit      # Security auditing
cargo install cargo-outdated   # Check for outdated dependencies
```

### Command Name

The main command is **`declarch`** (not `dcl`). 

```bash
# Build the binary
cargo build

# Run with cargo
cargo run -- sync preview --diff

# Or use the fish alias for development:
ddev sync preview --diff
```

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_sync_empty_config

# Check code
cargo check

# Run clippy
cargo clippy --all-targets

# Fix clippy warnings
cargo clippy --fix --lib

# Format code
cargo fmt

# Security audit
cargo audit

# Check outdated deps
cargo outdated
```

## Testing

### Test Structure

- **Unit tests**: 159 tests in src/
- **Integration tests**: 14 tests in tests/sync_integration_tests.rs
- **Doc tests**: 8 tests

### Running Tests

```bash
# All tests
cargo test

# Library tests only
cargo test --lib

# Integration tests only
cargo test --test sync_integration_tests

# CLI tests
cargo test --test cli_suite

# With output
cargo test -- --nocapture
```

## Code Style

### Conventions

1. **Error Handling**: Use `thiserror` for custom errors
2. **Logging**: Use standard `println!` with colored output via `colored`
3. **Documentation**: Document all public APIs with `///`
4. **Comments**: Minimal inline comments, self-documenting code
5. **Types**: Strong typing, avoid `unwrap()` in production code

### Commit Messages

Use conventional commits:

```
feat(scope): description
fix(scope): description
docs(scope): description
test(scope): description
refactor(scope): description
chore(scope): description
```

## Key Features

### New Flags (Implemented)

1. **`declarch sync preview --diff`**: Git-diff-like output
2. **`declarch info status --summary`**: Quick status overview

### Critical Fixes (Completed)

1. Pre-sync hooks now execute correctly
2. State properly saved after sync
3. Transaction plan displayed before execution
4. State file permissions set to 0600

## Security

- State files: 0600 permissions (owner read/write only)
- Hook validation: Restricted character set
- Input sanitization: Prevents shell injection
- File locking: Prevents concurrent state corruption

## Performance

- Zero-copy where possible
- Lazy evaluation for expensive operations
- File locking with fs2 for concurrent access
- Atomic state updates with temp files

## TODO

- [ ] Add more integration tests for edge cases
- [ ] Implement cargo-audit/cargo-outdated in CI
- [ ] Improve error messages with suggestions
- [ ] Add performance benchmarks

## Resources

- **Documentation**: https://nixval.github.io/declarch/
- **Issues**: https://github.com/nixval/declarch/issues
- **License**: MIT
