# Declarch Test Suite

## Directory Structure

```
tests/
├── unit/           # Unit tests for individual modules
├── integration/    # Integration tests for workflows
└── fixtures/       # Test fixtures and sample data
    ├── configs/    # Sample KDL configurations
    └── states/     # Sample state.json files
```

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test unit_

# Run only integration tests
cargo test --test integration_

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_state_locking
```

## Test Categories

### Unit Tests (`unit/`)
- `state_io_tests.rs` - State locking, save/load, backup rotation
- `matcher_tests.rs` - Package matching logic
- `parser_tests.rs` - KDL parsing

### Integration Tests (`integration/`)
- `sync_flow_tests.rs` - Full sync workflow
- `backend_tests.rs` - Backend implementations
- `install_tests.rs` - Install command workflow

## Fixtures

### Configs (`fixtures/configs/`)
Sample KDL configurations for testing:
- `minimal.kdl` - Minimal valid config
- `desktop.kdl` - Full desktop setup
- `conflicts.kdl` - Config with conflicts

### States (`fixtures/states/`)
Sample state.json files:
- `empty_state.json` - Fresh install state
- `populated_state.json` - State with packages
- `outdated_state.json` - State needing migration
