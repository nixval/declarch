# Tests

Current test files in this repository:

```text
tests/
├── backend_parsers.rs
├── cli_suite.rs
├── smart_matching.rs
├── state_restore.rs
├── test_custom_backends.rs
└── unit/
    └── state_io_tests.rs
```

## Run tests

```bash
# all tests
cargo test

# one file
cargo test --test cli_suite

# one unit module
cargo test state_io_tests

# verbose output
cargo test -- --nocapture
```

## Notes

- Integration-style coverage is mostly in top-level `tests/*.rs`.
- Focus areas: parser correctness, CLI behavior, state safety, and custom backend support.
